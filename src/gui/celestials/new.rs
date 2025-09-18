use super::{
    PreviewBody, SimState, Universe, UniverseId, declare_id,
    info::body_window_info,
    selectable_body_tree,
    units::{AutoUnit, UnitEnum, length::LengthUnit, mass::MassUnit},
};
use float_pretty_print::PrettyPrintFloat;
use keplerian_sim::{Orbit, OrbitTrait};
use strum::IntoEnumIterator;
use three_d::egui::{
    Align, Color32, ComboBox, Context, DragValue, Grid, Label, Layout, PopupCloseBehavior,
    RichText, Slider, TextEdit, TextWrapMode, Ui, Window, color_picker::color_edit_button_srgb,
};

declare_id!(salt_only, NEW_BODY_PHYS, b"Creation");
declare_id!(salt_only, NEW_BODY_ORBIT, b"3111ptic");
declare_id!(salt_only, DRAG_VALUE_WITH_UNIT_PREFIX, b"2ParSecs");
declare_id!(salt_only, NEW_BODY_MASS, b"nMa551ve");
declare_id!(salt_only, NEW_BODY_RADIUS, b"extraRad");
declare_id!(salt_only, NEW_BODY_PERIAPSIS, b"TOOcl0se");
declare_id!(salt_only, NEW_BODY_PARENT_COMBO_BOX, b"dr0pChld");
declare_id!(NEW_BODY_PARENT_TREE, b"treeL1K3");
declare_id!(salt_only, NEW_BODY_INFO_GRID, b"NEEEERD!");

pub(in super::super) struct NewBodyWindowState {
    mass_unit: AutoUnit<MassUnit>,
    radius_unit: AutoUnit<LengthUnit>,
    periapsis_unit: AutoUnit<LengthUnit>,
    pub(super) request_focus: bool,
}

impl Default for NewBodyWindowState {
    fn default() -> Self {
        Self {
            mass_unit: AutoUnit {
                auto: true,
                unit: MassUnit::Kilograms,
            },
            radius_unit: AutoUnit {
                auto: true,
                unit: LengthUnit::Meters,
            },
            periapsis_unit: AutoUnit {
                auto: true,
                unit: LengthUnit::Meters,
            },
            request_focus: true,
        }
    }
}

pub(super) fn new_body_window(ctx: &Context, sim_state: &mut SimState) {
    let mut wrapper = match sim_state.preview_body.take() {
        Some(w) => Some(w),
        None => {
            sim_state.ui.new_body_window_state = None;
            return;
        }
    };

    let window_state = sim_state.ui.new_body_window_state.get_or_insert_default();

    let mut open = true;

    let window = Window::new("New Body")
        .scroll([false, true])
        .resizable([false, true])
        .default_width(300.0)
        .min_width(300.0)
        .max_width(300.0)
        .min_height(200.0)
        .open(&mut open)
        .show(ctx, |ui| {
            let wrapper = match wrapper.take() {
                Some(w) => w,
                None => return,
            };
            ui.scope(|ui| {
                sim_state.preview_body =
                    new_body_window_content(ui, &mut sim_state.universe, wrapper, window_state);
            });
        });

    if let Some(w) = wrapper {
        sim_state.preview_body = Some(w);
    }

    if !open {
        sim_state.preview_body = None;
    }

    if let Some(w) = window
        && window_state.request_focus
    {
        w.response.request_focus();

        if w.response.has_focus() {
            window_state.request_focus = false;
        }
    }
}

fn new_body_window_content(
    ui: &mut Ui,
    universe: &mut Universe,
    mut wrapper: PreviewBody,
    window_state: &mut NewBodyWindowState,
) -> Option<PreviewBody> {
    ui.visuals_mut().override_text_color = Some(Color32::WHITE);

    let text = RichText::new("Physical Characteristics")
        .underline()
        .size(16.0);
    let label = Label::new(text);
    ui.add(label);
    ui.add_space(8.0);
    Grid::new(NEW_BODY_PHYS_SALT)
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            new_body_window_phys(ui, &mut wrapper, window_state)
        });

    let text = RichText::new("Orbital Parameters").underline().size(16.0);
    let label = Label::new(text);
    ui.add_space(12.0);
    ui.add(label);
    ui.add_space(8.0);
    Grid::new(NEW_BODY_ORBIT_SALT)
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            new_body_window_orbit(
                ui,
                &mut wrapper.body.orbit,
                &mut wrapper.parent_id,
                universe,
                window_state,
            )
        });

    ui.add_space(12.0);

    let derived_info = RichText::new("Derived Information")
        .color(Color32::WHITE)
        .size(16.0)
        .underline();

    ui.collapsing(derived_info, |ui| {
        ui.set_min_width(ui.available_width());
        Grid::new(NEW_BODY_INFO_GRID_SALT)
            .num_columns(2)
            // .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                body_window_info(ui, &wrapper.body, wrapper.parent_id, universe);
            });
    });

    ui.add_space(16.0);
    if ui.button("Confirm").clicked() {
        let _ = universe.add_body(wrapper.body, wrapper.parent_id);
        return None;
    }

    return Some(wrapper);
}

fn new_body_window_phys(
    ui: &mut Ui,
    wrapper: &mut PreviewBody,
    window_state: &mut NewBodyWindowState,
) {
    // TODO: Hover popups
    ui.label("Body name");
    ui.add(
        TextEdit::singleline(&mut wrapper.body.name)
            .char_limit(255)
            .hint_text("Enter new body name")
            .desired_width(f32::INFINITY),
    );
    ui.end_row();

    ui.label("Body color");
    let original_srgb: [u8; 3] = wrapper.body.color.into();
    let mut srgb = original_srgb.clone();
    color_edit_button_srgb(ui, &mut srgb);
    if srgb != original_srgb {
        wrapper.body.color = srgb.into();
    }
    ui.end_row();

    ui.label("Mass");
    drag_value_with_unit(
        NEW_BODY_MASS_SALT,
        ui,
        &mut wrapper.body.mass,
        &mut window_state.mass_unit,
    );
    ui.end_row();

    ui.label("Radius");
    drag_value_with_unit(
        NEW_BODY_RADIUS_SALT,
        ui,
        &mut wrapper.body.radius,
        &mut window_state.radius_unit,
    );
    ui.end_row();
}

fn new_body_window_orbit(
    ui: &mut Ui,
    orbit: &mut Option<Orbit>,
    parent_id: &mut Option<UniverseId>,
    universe: &Universe,
    window_state: &mut NewBodyWindowState,
) {
    // TODO: Hover popups
    ui.label("Parent body");
    ComboBox::from_id_salt(NEW_BODY_PARENT_COMBO_BOX_SALT)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .wrap_mode(TextWrapMode::Extend)
        .selected_text(
            parent_id
                .map(|parent_id| universe.get_body(parent_id))
                .flatten()
                .map(|w| &*w.body.name)
                .unwrap_or("—"),
        )
        .show_ui(ui, |ui| {
            selectable_body_tree(ui, *NEW_BODY_PARENT_TREE_ID, universe, parent_id);
        });
    ui.end_row();

    let parent_id = match parent_id {
        Some(id) => *id,
        None => return,
    };
    let orbit = orbit.get_or_insert_with(|| {
        let (periapsis, mu) = universe
            .get_body(parent_id)
            .map(|w| {
                (
                    w.body.radius * 2.0,
                    w.body.mass * universe.get_gravitational_constant(),
                )
            })
            .unwrap_or((2.0, 1.0));
        Orbit::new(0.0, periapsis, 0.0, 0.0, 0.0, 0.0, mu)
    });

    ui.label("Eccentricity");
    let mut eccentricity = orbit.get_eccentricity();
    let dv = DragValue::new(&mut eccentricity)
        .range(0.0..=f64::MAX)
        .speed(0.01);
    let dv = ui.add_sized((ui.available_width(), 18.0), dv);
    if dv.changed() {
        orbit.set_eccentricity(eccentricity);
    }
    ui.end_row();

    ui.label("Periapsis");
    let mut periapsis = orbit.get_periapsis();
    drag_value_with_unit(
        NEW_BODY_PERIAPSIS_SALT,
        ui,
        &mut periapsis,
        &mut window_state.periapsis_unit,
    );
    if periapsis != orbit.get_periapsis() {
        orbit.set_periapsis(periapsis);
    }
    ui.end_row();

    ui.label("Inclination");
    let mut inclination = orbit.get_inclination().to_degrees();
    let slider = Slider::new(&mut inclination, 0.0..=180.0).suffix('°');
    let slider = ui.add_sized((ui.available_width(), 18.0), slider);
    if slider.changed() {
        orbit.set_inclination(inclination.to_radians());
    }
    ui.end_row();

    ui.label("Arg. of Pe.");
    let mut arg_pe = orbit.get_arg_pe().to_degrees();
    let slider = Slider::new(&mut arg_pe, 0.0..=360.0).suffix('°');
    let slider = ui.add(slider);
    if slider.changed() {
        orbit.set_arg_pe(arg_pe.to_radians());
    }
    ui.end_row();

    ui.label("RAAN");
    let mut lan = orbit.get_long_asc_node().to_degrees();
    let slider = Slider::new(&mut lan, 0.0..=360.0).suffix('°');
    let slider = ui.add(slider);
    if slider.changed() {
        orbit.set_long_asc_node(lan.to_radians());
    }
    ui.end_row();

    let mut mean_anomaly = orbit.get_mean_anomaly_at_epoch().to_degrees();
    if orbit.get_eccentricity() < 1.0 {
        ui.label("Mean anom.");
        let slider = Slider::new(&mut mean_anomaly, 0.0..=360.0).suffix('°');
        let slider = ui.add(slider);
        if mean_anomaly < 0.0 || mean_anomaly > 360.0 {
            mean_anomaly = mean_anomaly.rem_euclid(360.0);
        }
        if slider.changed() {
            orbit.set_mean_anomaly_at_epoch(mean_anomaly.to_radians());
        }
    } else {
        ui.label("Hyp. m. anom.");
        let dv = DragValue::new(&mut mean_anomaly)
            .range(f64::MIN..=f64::MAX)
            .suffix('°');
        let dv = ui.add(dv);
        if dv.changed() {
            orbit.set_mean_anomaly_at_epoch(mean_anomaly.to_radians());
        }
    }
    ui.end_row();
}

fn drag_value_with_unit<'a, U>(
    id_salt: impl std::hash::Hash,
    ui: &mut Ui,
    base_val: &'a mut f64,
    unit: &'a mut AutoUnit<U>,
) where
    U: UnitEnum,
{
    ui.scope(|ui| {
        ui.set_width(ui.available_width());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            drag_value_with_unit_inner(id_salt, ui, base_val, unit)
        });
    });
}

fn drag_value_with_unit_inner<'a, U>(
    id_salt: impl std::hash::Hash,
    ui: &mut Ui,
    base_val: &'a mut f64,
    unit: &'a mut AutoUnit<U>,
) where
    U: UnitEnum,
{
    let unit_scale = unit.get_value();
    let mut scaled_val = *base_val / unit_scale;
    let speed = scaled_val * 4e-3;
    let dv = DragValue::new(&mut scaled_val)
        .custom_formatter(|num, _| format!("{:3.8}", PrettyPrintFloat(num)))
        .range(f64::MIN_POSITIVE..=f64::MAX)
        .speed(speed);
    let cb = ComboBox::from_id_salt((DRAG_VALUE_WITH_UNIT_PREFIX_SALT, id_salt))
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .selected_text(unit.unit.to_string());

    cb.show_ui(ui, |ui: &mut Ui| {
        for unit_variant in <U as IntoEnumIterator>::iter() {
            let button = ui.selectable_label(unit_variant == **unit, unit_variant.to_string());

            if button.clicked() {
                unit.unit = unit_variant;
                unit.auto = false;
            }
        }
        ui.separator();
        let button = ui.selectable_label(unit.auto, "Auto-pick");
        if button.clicked() {
            unit.auto ^= true;
        }
    });

    let dv = ui.add_sized([ui.available_width(), 18.0], dv);

    if dv.changed() {
        *base_val = scaled_val * unit_scale;
    }

    if !dv.dragged() && !dv.has_focus() {
        unit.update(*base_val);
    }
}
