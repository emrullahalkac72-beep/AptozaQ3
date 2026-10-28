//! Aptoza koyu/açık tema renkleri ve egui stili.
use egui::{Color32, FontFamily, FontId, Stroke, TextStyle};
use std::sync::atomic::{AtomicBool, Ordering};

static DARK: AtomicBool = AtomicBool::new(true);

pub fn set_dark(dark: bool) {
    DARK.store(dark, Ordering::Relaxed);
}

pub fn is_dark() -> bool {
    DARK.load(Ordering::Relaxed)
}

pub fn bg() -> Color32 {
    if is_dark() {
        Color32::from_rgb(0x0d, 0x11, 0x17)
    } else {
        Color32::from_rgb(0xf3, 0xf5, 0xf8)
    }
}

pub fn card() -> Color32 {
    if is_dark() {
        Color32::from_rgb(0x16, 0x1b, 0x22)
    } else {
        Color32::from_rgb(0xff, 0xff, 0xff)
    }
}

pub fn card2() -> Color32 {
    if is_dark() {
        Color32::from_rgb(0x1c, 0x21, 0x28)
    } else {
        Color32::from_rgb(0xec, 0xf0, 0xf5)
    }
}

pub fn border() -> Color32 {
    if is_dark() {
        Color32::from_rgb(0x30, 0x36, 0x3d)
    } else {
        Color32::from_rgb(0xd0, 0xd7, 0xde)
    }
}

pub fn txt() -> Color32 {
    if is_dark() {
        Color32::from_rgb(0xe6, 0xed, 0xf3)
    } else {
        Color32::from_rgb(0x1f, 0x23, 0x28)
    }
}

pub fn txt2() -> Color32 {
    if is_dark() {
        Color32::from_rgb(0x8b, 0x94, 0x9e)
    } else {
        Color32::from_rgb(0x5b, 0x64, 0x6e)
    }
}

pub fn accent() -> Color32 {
    Color32::from_rgb(0x23, 0x86, 0x36)
}

pub fn accent2() -> Color32 {
    Color32::from_rgb(0x1f, 0x53, 0x8d)
}

pub fn red() -> Color32 {
    Color32::from_rgb(0xda, 0x36, 0x33)
}

pub fn yellow() -> Color32 {
    Color32::from_rgb(0xd2, 0x99, 0x22)
}

pub fn purple() -> Color32 {
    Color32::from_rgb(0x8b, 0x5c, 0xf6)
}

pub fn green() -> Color32 {
    Color32::from_rgb(0x2e, 0xa0, 0x43)
}

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut v = if is_dark() {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    v.panel_fill = bg();
    v.window_fill = card();
    v.extreme_bg_color = if is_dark() {
        Color32::from_rgb(0x08, 0x0c, 0x10)
    } else {
        Color32::from_rgb(0xf8, 0xf9, 0xfb)
    };
    v.faint_bg_color = card2();
    v.widgets.noninteractive.bg_fill = card();
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, txt2());
    v.widgets.inactive.bg_fill = card2();
    v.widgets.inactive.weak_bg_fill = card2();
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, txt());
    v.widgets.hovered.bg_fill = border();
    v.widgets.hovered.weak_bg_fill = border();
    v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, txt());
    v.widgets.active.bg_fill = accent2();
    v.widgets.active.weak_bg_fill = accent2();
    v.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    v.selection.bg_fill = accent2();
    v.selection.stroke = Stroke::new(1.0_f32, txt());
    v.hyperlink_color = Color32::from_rgb(0x58, 0xa6, 0xff);
    style.visuals = v;
    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.interact_size.y = 25.0;
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(11.0));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(14.0));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(14.0));
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(19.0));
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(14.0, FontFamily::Monospace),
    );
    ctx.set_style(style);
}

pub fn card_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(card())
        .stroke(Stroke::new(1.0_f32, border()))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(14))
}

pub fn dark_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(card2())
        .stroke(Stroke::new(1.0_f32, border()))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(10))
}
