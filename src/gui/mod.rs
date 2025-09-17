use std::{collections::HashMap, sync::Arc};

use super::sim::universe::{BulkMuSetterMode, Id as UniverseId, Universe};
pub(crate) use celestials::PreviewBody;
use glam::DVec3;
use ordered_float::NotNan;
use three_d::{
    Context as ThreeDContext, Event as ThreeDEvent, GUI, Viewport,
    egui::{
        self, Context as EguiContext, FontData, FontFamily, FontId, Vec2,
        epaint::text::{FontInsert, FontPriority, InsertFontFamily},
    },
};

mod bottom_bar;
mod celestials;
mod fps;

macro_rules! declare_id {
    (salt_only, $name:ident, $val:expr) => {
        ::pastey::paste! {
            const [<$name _SALT>]: ::core::num::NonZeroU64 =
                ::core::num::NonZeroU64::new(u64::from_be_bytes(*$val)).unwrap();
        }
    };
    ($name:ident, $val:expr) => {
        ::pastey::paste! {
            const [<$name _SALT>]: ::core::num::NonZeroU64 =
                ::core::num::NonZeroU64::new(u64::from_be_bytes(*$val)).unwrap();
            const [<$name _ID>]: ::std::sync::LazyLock<::three_d::egui::Id> =
                ::std::sync::LazyLock::new(|| ::three_d::egui::Id::new([<$name _SALT>]));
        }
    };
}
use declare_id;

const MIN_TOUCH_TARGET_LEN: f32 = 48.0;
const MIN_TOUCH_TARGET_VEC: Vec2 = Vec2::splat(MIN_TOUCH_TARGET_LEN);

struct UiState {
    bottom_bar_state: bottom_bar::BottomBarState,
    frame_data: fps::FrameData,
    body_list_window_state: celestials::list::BodyListWindowState,
    new_body_window_state: Option<celestials::new::NewBodyWindowState>,
    edit_body_window_state: celestials::edit::EditBodyWindowState,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            bottom_bar_state: bottom_bar::BottomBarState::default(),
            frame_data: fps::FrameData::new(),
            body_list_window_state: celestials::list::BodyListWindowState::default(),
            new_body_window_state: None,
            edit_body_window_state: celestials::edit::EditBodyWindowState::default(),
        }
    }
}

pub(crate) struct SimState {
    pub universe: Universe,
    pub mu_setter_mode: BulkMuSetterMode,
    pub sim_speed: f64,
    pub running: bool,
    focused_body: UniverseId,
    pub focus_offset: DVec3,
    pub preview_body: Option<celestials::PreviewBody>,
    ui: UiState,
}

impl SimState {
    pub(crate) fn new(universe: Universe) -> Self {
        Self {
            universe,
            ..Default::default()
        }
    }
    pub(crate) fn switch_focus(
        &mut self,
        focus_body_id: UniverseId,
        position_map: &HashMap<UniverseId, DVec3>,
    ) {
        // old_position → old_focus
        //            \      ↓
        //             \new_focus

        let old_focus = *position_map.get(&self.focused_body).unwrap_or(&DVec3::ZERO);
        let old_position = old_focus + self.focus_offset;
        let new_focus = *position_map.get(&focus_body_id).unwrap_or(&DVec3::ZERO);
        let new_offset = old_position - new_focus;
        self.focused_body = focus_body_id;
        self.focus_offset = if new_offset.is_nan() {
            DVec3::ZERO
        } else {
            new_offset
        };
    }
    #[inline]
    pub(crate) fn focused_body(&self) -> UniverseId {
        self.focused_body
    }
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            universe: Universe::default(),
            mu_setter_mode: BulkMuSetterMode::default(),
            sim_speed: 1.0,
            running: true,
            focused_body: 0,
            focus_offset: DVec3::ZERO,
            preview_body: None,
            ui: UiState::default(),
        }
    }
}

pub(super) fn create(context: &ThreeDContext) -> GUI {
    let gui = GUI::new(context);
    egui_extras::install_image_loaders(gui.context());
    gui.context().style_mut(|styles| {
        styles.text_styles.insert(
            egui::TextStyle::Name(Arc::clone(
                &*bottom_bar::TIME_SPEED_DRAG_VALUE_TEXT_STYLE_NAME,
            )),
            FontId::monospace(16.0),
        );
    });
    gui.context().add_font(FontInsert {
        name: String::from("DejaVuSans"),
        data: FontData::from_static(include_bytes!(
            "../assets/deja_vu_sans/DejaVuSans-subset.ttf"
        )),
        families: vec![InsertFontFamily {
            family: FontFamily::Proportional,
            priority: FontPriority::Lowest,
        }],
    });
    gui
}

pub(super) fn update(
    gui: &mut GUI,
    sim_state: &mut SimState,
    events: &mut Vec<ThreeDEvent>,
    accumulated_time_ms: f64,
    viewport: Viewport,
    device_pixel_ratio: f32,
    elapsed_time: f64,
    position_map: &HashMap<UniverseId, DVec3>,
) -> bool {
    if let Ok(frame_duration) = NotNan::new(elapsed_time / 1000.0)
        && frame_duration.is_finite()
    {
        sim_state.ui.frame_data.insert_frame_data(frame_duration);
    }
    gui.update(
        events,
        accumulated_time_ms,
        viewport,
        device_pixel_ratio,
        |ctx| handle_ui(ctx, elapsed_time, sim_state, position_map),
    )
}

fn handle_ui(
    ctx: &EguiContext,
    elapsed_time: f64,
    sim_state: &mut SimState,
    position_map: &HashMap<UniverseId, DVec3>,
) {
    fps::fps_area(ctx, &sim_state.ui.frame_data);
    bottom_bar::draw(ctx, sim_state, elapsed_time);
    celestials::celestial_windows(ctx, sim_state, position_map);
}
