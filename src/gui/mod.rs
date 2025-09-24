use std::{collections::HashMap, sync::Arc};

use super::sim::universe::{BulkMuSetterMode, Id as UniverseId, Universe};
pub(crate) use celestials::PreviewBody;
use glam::DVec3;
use ordered_float::NotNan;
use three_d::{
    Context as ThreeDContext, Event as ThreeDEvent, GUI, Viewport,
    egui::{
        self, Context as EguiContext, CursorIcon, FontData, FontFamily, FontId, OpenUrl,
        OutputCommand, Vec2,
        epaint::text::{FontInsert, FontPriority, InsertFontFamily},
    },
};

mod about;
mod bottom_bar;
mod celestials;
mod fps;
mod unit_dv;
mod welcome;

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
    welcome_window_state: welcome::WindowState,
    is_about_window_open: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            bottom_bar_state: bottom_bar::BottomBarState::default(),
            frame_data: fps::FrameData::new(),
            body_list_window_state: celestials::list::BodyListWindowState::default(),
            new_body_window_state: None,
            edit_body_window_state: celestials::edit::EditBodyWindowState::default(),
            welcome_window_state: welcome::WindowState::default(),
            is_about_window_open: false,
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
    welcome::draw(ctx, &mut sim_state.ui.welcome_window_state);
    bottom_bar::draw(ctx, sim_state, elapsed_time);
    celestials::celestial_windows(ctx, sim_state, position_map);
    about::draw(ctx, &mut sim_state.ui);
    ctx.output(|output| {
        for command in &output.commands {
            handle_command(&command);
        }
        set_cursor_icon(output.cursor_icon);
    });
}

#[cfg(target_family = "wasm")]
const fn cursor_icon_to_css_value(cursor: CursorIcon) -> &'static str {
    match cursor {
        CursorIcon::Default => "default",
        CursorIcon::None => "none",
        CursorIcon::PointingHand => "pointer",
        CursorIcon::Text => "text",
        CursorIcon::VerticalText => "vertical-text",
        CursorIcon::Crosshair => "crosshair",
        CursorIcon::Move => "move",
        CursorIcon::Grab => "grab",
        CursorIcon::Grabbing => "grabbing",
        CursorIcon::Help => "help",
        CursorIcon::Progress => "progress",
        CursorIcon::Wait => "wait",
        CursorIcon::NotAllowed => "not-allowed",
        CursorIcon::NoDrop => "no-drop",
        CursorIcon::AllScroll => "all-scroll",
        CursorIcon::ResizeHorizontal => "ew-resize",
        CursorIcon::ResizeVertical => "ns-resize",
        CursorIcon::ResizeNwSe => "nwse-resize",
        CursorIcon::ResizeNeSw => "nesw-resize",
        CursorIcon::ZoomIn => "zoom-in",
        CursorIcon::ZoomOut => "zoom-out",
        CursorIcon::Copy => "copy",
        CursorIcon::Alias => "alias",
        CursorIcon::ContextMenu => "context-menu",
        CursorIcon::Cell => "cell",
        CursorIcon::ResizeEast => "e-resize",
        CursorIcon::ResizeSouthEast => "se-resize",
        CursorIcon::ResizeSouth => "s-resize",
        CursorIcon::ResizeSouthWest => "sw-resize",
        CursorIcon::ResizeWest => "w-resize",
        CursorIcon::ResizeNorthWest => "nw-resize",
        CursorIcon::ResizeNorth => "n-resize",
        CursorIcon::ResizeNorthEast => "ne-resize",
        CursorIcon::ResizeColumn => "col-resize",
        CursorIcon::ResizeRow => "row-resize",
    }
}

fn set_cursor_icon(cursor: CursorIcon) {
    #[cfg(target_family = "wasm")]
    {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(body) = document.body() else {
            return;
        };

        let _ = body
            .style()
            .set_property("cursor", cursor_icon_to_css_value(cursor));
    }
    #[cfg(not(target_family = "wasm"))]
    {
        // TODO: Setting cursor icon on native
        let _ = cursor;
    }
}

fn handle_command(command: &OutputCommand) {
    match command {
        OutputCommand::CopyText(text) => copy_text(&text),
        OutputCommand::CopyImage(_) => eprintln!("Copying images is not implemented."),
        OutputCommand::OpenUrl(url) => open_url(url),
    }
}
fn copy_text(text: &str) {
    #[cfg(target_family = "wasm")]
    {
        use wasm_bindgen::JsCast;
        let document = match web_sys::window().and_then(|window| window.document()) {
            Some(d) => d,
            None => return,
        };
        let html_document = match document.clone().dyn_into::<web_sys::HtmlDocument>() {
            Ok(d) => d,
            Err(_) => return,
        };
        let body = match document.body() {
            Some(b) => b,
            None => return,
        };
        let textarea = match document
            .create_element("textarea")
            .and_then(|el| Ok(el.dyn_into::<web_sys::HtmlTextAreaElement>()))
        {
            Ok(Ok(ta)) => ta,
            Ok(Err(_)) => return,
            Err(_) => return,
        };

        textarea.set_value(text);
        let _ = textarea.style().set_property("position", "fixed");
        let _ = textarea.style().set_property("left", "-9999vw");
        let _ = textarea.style().set_property("width", "0");
        let _ = body.append_child(&textarea);
        let _ = textarea.select();
        let _ = html_document.exec_command("copy");
        let _ = body.remove_child(&textarea);
    }
    #[cfg(not(target_family = "wasm"))]
    {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            if let Err(e) = cb.set_text(text.to_owned()) {
                eprintln!("Failed to set clipboard text: {e}");
            }
        } else {
            eprintln!("Failed to open clipboard");
        }
    }
}
fn open_url(command: &OpenUrl) {
    #[cfg(target_family = "wasm")]
    web_sys::window()
        .map(|w| w.open_with_url_and_target(&command.url, "_blank"))
        .expect("window should exist")
        .expect("url should be openable");
    #[cfg(not(target_family = "wasm"))]
    if let Err(e) = open::that_detached(&command.url) {
        eprintln!("Failed to open URL '{}': {e}", &command.url);
    }
}
