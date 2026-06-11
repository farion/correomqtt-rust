use egui::{Color32, CornerRadius, Frame, Modal};

use crate::theme::ThemeTokens;

pub(crate) const SCRIM_ALPHA: u8 = 176;
const MODAL_RADIUS: u8 = 4;
const MODAL_PADDING: i8 = 16;

pub(crate) fn frame(tokens: ThemeTokens) -> Frame {
    let bg = tokens.window_bg;
    Frame::NONE
        .fill(Color32::from_rgb(bg.r(), bg.g(), bg.b()))
        .corner_radius(CornerRadius::same(MODAL_RADIUS))
        .inner_margin(egui::Margin::same(MODAL_PADDING))
}

pub(crate) fn style(modal: Modal, tokens: ThemeTokens) -> Modal {
    modal
        .frame(frame(tokens))
        .backdrop_color(Color32::from_black_alpha(SCRIM_ALPHA))
}
