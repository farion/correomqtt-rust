use egui::Ui;

pub(crate) fn right_aligned_text(ui: &Ui, pos: egui::Pos2, text: &str, color: egui::Color32) {
    ui.painter().text(
        pos,
        egui::Align2::RIGHT_TOP,
        text,
        egui::TextStyle::Small.resolve(ui.style()),
        color,
    );
}

pub(crate) fn truncated_text(
    ui: &mut Ui,
    pos: egui::Pos2,
    width: f32,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
) {
    if width <= 0.0 {
        return;
    }
    let mut job = egui::text::LayoutJob::simple(text.to_owned(), font, color, width);
    job.wrap = egui::text::TextWrapping::truncate_at_width(width);
    job.break_on_newline = false;
    let galley = ui.fonts(|fonts| fonts.layout_job(job));
    ui.painter().galley(pos, galley, color);
}

pub(crate) fn middle_ellipsis(ui: &Ui, text: &str, font: egui::FontId, width: f32) -> String {
    if width <= 0.0 {
        return String::new();
    }
    if text_width(ui, text, font.clone()) <= width {
        return text.to_owned();
    }
    let ellipsis = "...";
    if text_width(ui, ellipsis, font.clone()) > width {
        return String::new();
    }

    let chars = text.chars().collect::<Vec<_>>();
    let mut prefix_len = chars.len().div_ceil(2);
    let mut suffix_len = chars.len() / 2;
    while prefix_len + suffix_len > 0 {
        let candidate = format!(
            "{}{}{}",
            chars.iter().take(prefix_len).collect::<String>(),
            ellipsis,
            chars
                .iter()
                .skip(chars.len().saturating_sub(suffix_len))
                .collect::<String>()
        );
        if text_width(ui, &candidate, font.clone()) <= width {
            return candidate;
        }
        if prefix_len > suffix_len && prefix_len > 0 {
            prefix_len -= 1;
        } else {
            suffix_len = suffix_len.saturating_sub(1);
        }
    }
    ellipsis.to_owned()
}

pub(crate) fn formatted_size(bytes: usize) -> String {
    const UNITS: [&str; 5] = ["b", "kb", "mb", "gb", "tb"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else if value >= 10.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub(crate) fn text_width(ui: &Ui, text: &str, font: egui::FontId) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_owned(), font, egui::Color32::WHITE)
        .size()
        .x
}
