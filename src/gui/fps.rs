use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
};

use ordered_float::NotNan;
use three_d::egui::{Area, Color32, Context, FontId, Label, RichText, TextWrapMode, Ui};

use crate::gui::declare_id;

declare_id!(FPS_AREA, b"PerfArea");

pub(super) struct FrameData {
    frame_len_secs: VecDeque<NotNan<f64>>,
}

impl FrameData {
    const WINDOW_SIZE: usize = 1200;

    pub(super) fn new() -> Self {
        Self {
            frame_len_secs: VecDeque::with_capacity(Self::WINDOW_SIZE),
        }
    }

    /// Returns NaN if no frames recorded yet
    fn get_average_fps(&self) -> f64 {
        self.frame_len_secs.len() as f64 / *self.frame_len_secs.iter().copied().sum::<NotNan<f64>>()
    }

    /// Gets the 1% lows of FPS in the sliding window
    /// Returns NaN if no frames recorded yet
    fn get_low_average(&self) -> f64 {
        let data_amount = self.frame_len_secs.len() / 100;
        if data_amount == 0 {
            return f64::NAN;
        }

        let mut heap = BinaryHeap::with_capacity(data_amount + 1);

        for &time in &self.frame_len_secs {
            heap.push(Reverse(time));

            if heap.len() > data_amount {
                heap.pop();
            }
        }

        heap.len() as f64 / *heap.iter().map(|&x| x.0).sum::<NotNan<f64>>()
    }

    pub(super) fn insert_frame_data(&mut self, frame_duration: NotNan<f64>) {
        if self.frame_len_secs.capacity() == self.frame_len_secs.len() {
            self.frame_len_secs.pop_front();
        }

        self.frame_len_secs.push_back(frame_duration);
    }
}

pub(super) fn fps_area(ctx: &Context, frame_data: &FrameData) {
    let pos = 12.0;
    Area::new(*FPS_AREA_ID)
        .constrain_to(ctx.screen_rect())
        .fixed_pos((pos, pos))
        .default_width(1000.0)
        .show(&ctx, |ui| fps_inner(ui, frame_data));
}

fn fps_inner(ui: &mut Ui, frame_data: &FrameData) {
    let fps = frame_data.get_average_fps();
    let low = frame_data.get_low_average();

    let string = if low.is_nan() {
        format!("FPS: {fps:.0}")
    } else {
        format!("FPS: {fps:.0}\n1%L: {low:.0}")
    };
    const BACKGROUND_COLOR: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 128);
    let font = FontId::monospace(11.0);
    let text = RichText::new(string)
        .background_color(BACKGROUND_COLOR)
        .color(Color32::WHITE)
        .font(font);
    let label = Label::new(text)
        .wrap_mode(TextWrapMode::Extend)
        .selectable(false);
    ui.add(label);
}
