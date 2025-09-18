use float_pretty_print::PrettyPrintFloat;
use strum::IntoEnumIterator;
use three_d::egui::{Align, ComboBox, DragValue, Layout, PopupCloseBehavior, Ui};

use super::{
    super::units::{AutoUnit, UnitEnum},
    declare_id,
};

declare_id!(salt_only, DRAG_VALUE_WITH_UNIT_PREFIX, b"2ParSecs");

pub(super) fn drag_value_with_unit<'a, U>(
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
