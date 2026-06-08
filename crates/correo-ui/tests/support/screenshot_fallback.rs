use crate::screenshot_scenarios::{Capture, Scenario};
use correo_ui::theme;
use image::{Rgba, RgbaImage};

pub(super) fn fallback_shell_capture(capture: Capture) -> RgbaImage {
    let tokens = theme::static_tokens(capture.mode);
    let (width, height) = capture.size;
    let mut image = RgbaImage::from_pixel(width, height, px(tokens.window_bg));

    fill(&mut image, 0, 0, width, 28, tokens.panel_bg);
    stroke(&mut image, 0, 0, width, 28, tokens.border);
    fill(&mut image, 0, 28, width, 40, tokens.panel_bg);
    stroke(&mut image, 0, 28, width, 40, tokens.border);
    text_bar(&mut image, 16, 46, 126, tokens.text_primary);
    text_bar(&mut image, 168, 46, 110, tokens.success);
    draw_text(&mut image, 16, 38, "CONNECTIONS", tokens.text_primary, 2);
    draw_text(&mut image, 168, 38, "CONNECTED", tokens.success, 2);

    let content_bottom = height.saturating_sub(28);
    fill(
        &mut image,
        0,
        68,
        48,
        content_bottom.saturating_sub(68),
        tokens.panel_bg,
    );
    stroke(
        &mut image,
        0,
        68,
        48,
        content_bottom.saturating_sub(68),
        tokens.border,
    );
    for index in 0..6 {
        rail_button(&mut image, tokens, 8, 76 + index * 40, index == 0);
    }

    fill(
        &mut image,
        48,
        68,
        260,
        content_bottom.saturating_sub(68),
        tokens.panel_bg,
    );
    stroke(
        &mut image,
        48,
        68,
        260,
        content_bottom.saturating_sub(68),
        tokens.border,
    );
    text_bar(&mut image, 64, 96, 120, tokens.text_primary);
    draw_text(&mut image, 64, 88, "CONNECTIONS", tokens.text_primary, 2);
    for index in 0..4 {
        row(&mut image, tokens, 64, 132 + index * 64, index == 0);
    }
    let main_x = 308;
    let main_w = width.saturating_sub(main_x);
    let narrow_workbench = capture.size.0 <= 1024;
    match capture.scenario {
        Scenario::Launcher => fallback_launcher(&mut image, tokens, main_x, 88, main_w),
        Scenario::Workbench => {
            fallback_workbench(&mut image, tokens, main_x, 88, main_w, narrow_workbench)
        }
        Scenario::Scripts => fallback_scripts(&mut image, tokens, main_x, 88, main_w),
        _ => fallback_settings(&mut image, tokens, main_x, 88, main_w),
    }

    fill(&mut image, 0, content_bottom, width, 28, tokens.panel_bg);
    stroke(&mut image, 0, content_bottom, width, 28, tokens.border);
    text_bar(&mut image, 18, content_bottom + 14, 220, tokens.warning);
    draw_text(
        &mut image,
        18,
        content_bottom + 8,
        "DIAGNOSTICS",
        tokens.warning,
        2,
    );
    image
}

fn fallback_launcher(image: &mut RgbaImage, tokens: theme::ThemeTokens, x: u32, y: u32, w: u32) {
    panel(image, tokens, x + 20, y, w.saturating_sub(40), 120);
    text_bar(image, x + 38, y + 34, 180, tokens.text_primary);
    draw_text(
        image,
        x + 38,
        y + 24,
        "CONNECTION LAUNCHER",
        tokens.text_primary,
        2,
    );
    text_bar(
        image,
        x + 38,
        y + 72,
        w.saturating_sub(180),
        tokens.text_secondary,
    );
    button(image, tokens, x + w.saturating_sub(170), y + 22, 132, 30);
    draw_text(
        image,
        x + w.saturating_sub(156),
        y + 31,
        "ADD IMPORT",
        tokens.accent,
        1,
    );
    for index in 0..3 {
        let row_y = y + 148 + index * 116;
        panel(image, tokens, x + 20, row_y, w.saturating_sub(40), 92);
        text_bar(image, x + 42, row_y + 28, 146, tokens.text_primary);
        text_bar(image, x + 42, row_y + 52, 260, tokens.text_secondary);
        text_bar(
            image,
            x + w.saturating_sub(150),
            row_y + 36,
            88,
            tokens.success,
        );
    }
    draw_text(
        image,
        x + 42,
        y + 166,
        "LOCAL BROKER",
        tokens.text_primary,
        2,
    );
    draw_text(image, x + 42, y + 282, "QA TLS", tokens.text_primary, 2);
    draw_text(image, x + 42, y + 398, "STAGING", tokens.text_primary, 2);
}

fn fallback_workbench(
    image: &mut RgbaImage,
    tokens: theme::ThemeTokens,
    x: u32,
    y: u32,
    w: u32,
    narrow: bool,
) {
    panel(image, tokens, x + 20, y, w.saturating_sub(40), 58);
    text_bar(image, x + 38, y + 26, 140, tokens.text_primary);
    text_bar(image, x + 204, y + 26, 70, tokens.success);
    draw_text(
        image,
        x + 38,
        y + 16,
        "LOCAL BROKER",
        tokens.text_primary,
        2,
    );
    draw_text(
        image,
        x + 204,
        y + 16,
        "CONNECTED TLS UPTIME",
        tokens.success,
        2,
    );
    if narrow {
        panel(image, tokens, x + 20, y + 78, w.saturating_sub(40), 250);
        text_bar(image, x + 38, y + 102, 86, tokens.accent);
        draw_text(image, x + 38, y + 94, "PUBLISH SUBSCRIBE", tokens.accent, 2);
        draw_text(image, x + 42, y + 130, "SUBSCRIBE", tokens.text_primary, 2);
        for index in 0..4 {
            text_bar(
                image,
                x + 42,
                y + 146 + index * 38,
                w.saturating_sub(120),
                tokens.text_secondary,
            );
        }
    } else {
        let pane_w = w.saturating_sub(60) / 2;
        panel(image, tokens, x + 20, y + 78, pane_w, 260);
        panel(image, tokens, x + 40 + pane_w, y + 78, pane_w, 260);
        draw_text(image, x + 42, y + 98, "PUBLISH", tokens.text_primary, 2);
        draw_text(
            image,
            x + 62 + pane_w,
            y + 98,
            "SUBSCRIBE",
            tokens.text_primary,
            2,
        );
        for index in 0..5 {
            text_bar(
                image,
                x + 42,
                y + 120 + index * 36,
                pane_w.saturating_sub(80),
                tokens.text_secondary,
            );
            text_bar(
                image,
                x + 62 + pane_w,
                y + 120 + index * 36,
                pane_w.saturating_sub(80),
                tokens.text_secondary,
            );
        }
    }
    panel(image, tokens, x + 20, y + 360, w.saturating_sub(40), 120);
    text_bar(image, x + 38, y + 392, 180, tokens.text_primary);
    draw_text(
        image,
        x + 38,
        y + 382,
        "MESSAGE DETAIL",
        tokens.text_primary,
        2,
    );
}

fn fallback_scripts(image: &mut RgbaImage, tokens: theme::ThemeTokens, x: u32, y: u32, w: u32) {
    panel(image, tokens, x + 20, y, w.saturating_sub(40), 500);
    draw_text(image, x + 38, y + 20, "SCRIPTS", tokens.text_primary, 2);
    draw_text(
        image,
        x + 160,
        y + 20,
        "RUN CANCEL SAVE RENAME DELETE",
        tokens.accent,
        1,
    );
    panel(image, tokens, x + 38, y + 66, 260, 340);
    panel(image, tokens, x + 318, y + 66, w.saturating_sub(376), 340);
    draw_text(image, x + 54, y + 86, "FILES", tokens.text_primary, 2);
    for index in 0..4 {
        text_bar(
            image,
            x + 54,
            y + 122 + index * 54,
            180,
            tokens.text_primary,
        );
        text_bar(
            image,
            x + 54,
            y + 138 + index * 54,
            112,
            tokens.text_secondary,
        );
    }
    draw_text(image, x + 336, y + 86, "EDITOR", tokens.text_primary, 2);
    for index in 0..8 {
        text_bar(
            image,
            x + 336,
            y + 124 + index * 22,
            w.saturating_sub(460),
            tokens.text_secondary,
        );
    }
    draw_text(
        image,
        x + 336,
        y + 300,
        "INCREMENTAL LOG",
        tokens.warning,
        2,
    );
}

fn fallback_settings(image: &mut RgbaImage, tokens: theme::ThemeTokens, x: u32, y: u32, w: u32) {
    text_bar(image, x + 20, y + 24, 168, tokens.text_primary);
    draw_text(
        image,
        x + 20,
        y + 16,
        "CONNECTION SETTINGS",
        tokens.text_primary,
        2,
    );
    for index in 0..5 {
        text_bar(image, x + 20 + index * 108, y + 58, 72, tokens.accent);
    }
    draw_text(
        image,
        x + 20,
        y + 50,
        "MQTT TLS PROXY LWT ADV",
        tokens.accent,
        2,
    );
    panel(image, tokens, x + 20, y + 88, w.saturating_sub(40), 300);
    for index in 0..7 {
        text_bar(
            image,
            x + 42,
            y + 126 + index * 34,
            96,
            tokens.text_secondary,
        );
        text_bar(
            image,
            x + 178,
            y + 126 + index * 34,
            w.saturating_sub(260),
            tokens.text_primary,
        );
    }
    text_bar(image, x + 22, y + 424, 220, tokens.warning);
    draw_text(image, x + 42, y + 116, "NAME", tokens.text_secondary, 2);
    draw_text(image, x + 42, y + 150, "HOST", tokens.text_secondary, 2);
    draw_text(image, x + 42, y + 184, "PORT", tokens.text_secondary, 2);
    draw_text(
        image,
        x + 178,
        y + 116,
        "LOCAL BROKER",
        tokens.text_primary,
        2,
    );
    draw_text(
        image,
        x + 178,
        y + 150,
        "LOCAL...:1883",
        tokens.text_primary,
        2,
    );
    draw_text(
        image,
        x + 22,
        y + 414,
        "SAVE DISABLED VALIDATION",
        tokens.warning,
        2,
    );
    button(image, tokens, x + w.saturating_sub(230), y + 454, 70, 28);
    button(image, tokens, x + w.saturating_sub(150), y + 454, 58, 28);
    button(image, tokens, x + w.saturating_sub(82), y + 454, 58, 28);
    draw_text(
        image,
        x + w.saturating_sub(220),
        y + 462,
        "DISCARD",
        tokens.text_primary,
        1,
    );
    draw_text(
        image,
        x + w.saturating_sub(142),
        y + 462,
        "DELETE",
        tokens.text_primary,
        1,
    );
    draw_text(
        image,
        x + w.saturating_sub(72),
        y + 462,
        "SAVE",
        tokens.text_primary,
        1,
    );
}

fn rail_button(image: &mut RgbaImage, tokens: theme::ThemeTokens, x: u32, y: u32, selected: bool) {
    let fill_color = if selected {
        tokens.accent_selected_bg
    } else {
        tokens.panel_bg
    };
    fill(image, x, y, 32, 32, fill_color);
    stroke(image, x, y, 32, 32, tokens.border);
    if selected {
        fill(image, x, y, 3, 32, tokens.accent);
    }
}

fn row(image: &mut RgbaImage, tokens: theme::ThemeTokens, x: u32, y: u32, selected: bool) {
    let fill_color = if selected {
        tokens.accent_selected_bg
    } else {
        tokens.panel_raised
    };
    fill(image, x, y, 236, 52, fill_color);
    stroke(image, x, y, 236, 52, tokens.border);
    text_bar(image, x + 14, y + 18, 96, tokens.text_primary);
    text_bar(image, x + 14, y + 36, 140, tokens.text_secondary);
    let label = if selected { "LOCAL BROKER" } else { "QA TLS" };
    draw_text(image, x + 14, y + 12, label, tokens.text_primary, 1);
}

fn panel(image: &mut RgbaImage, tokens: theme::ThemeTokens, x: u32, y: u32, w: u32, h: u32) {
    fill(image, x, y, w, h, tokens.panel_bg);
    stroke(image, x, y, w, h, tokens.border);
}

fn button(image: &mut RgbaImage, tokens: theme::ThemeTokens, x: u32, y: u32, w: u32, h: u32) {
    fill(image, x, y, w, h, tokens.panel_raised);
    stroke(image, x, y, w, h, tokens.border);
}

fn text_bar(image: &mut RgbaImage, x: u32, y: u32, w: u32, color: egui::Color32) {
    fill(image, x, y, w, 3, color);
}

fn draw_text(image: &mut RgbaImage, x: u32, y: u32, text: &str, color: egui::Color32, scale: u32) {
    let scale = scale.max(1);
    let mut cursor = x;
    for character in text.chars() {
        if character.is_whitespace() {
            cursor = cursor.saturating_add(4 * scale);
            continue;
        }
        for (row, bits) in glyph(character.to_ascii_uppercase()).iter().enumerate() {
            for column in 0..3 {
                if bits & (1 << (2 - column)) != 0 {
                    fill(
                        image,
                        cursor + column * scale,
                        y + row as u32 * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }
        cursor = cursor.saturating_add(4 * scale);
    }
}

fn glyph(character: char) -> [u32; 5] {
    match character {
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        'C' => [0b011, 0b100, 0b100, 0b100, 0b011],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'G' => [0b011, 0b100, 0b101, 0b101, 0b011],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b101, 0b111, 0b111, 0b111, 0b101],
        'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
        'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
        'Q' => [0b010, 0b101, 0b101, 0b111, 0b011],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b110, 0b001, 0b010, 0b100, 0b111],
        '3' => [0b110, 0b001, 0b010, 0b001, 0b110],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b110, 0b001, 0b110],
        '6' => [0b011, 0b100, 0b110, 0b101, 0b010],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b010, 0b101, 0b010, 0b101, 0b010],
        '9' => [0b010, 0b101, 0b011, 0b001, 0b110],
        ':' => [0, 0b010, 0, 0b010, 0],
        '.' => [0, 0, 0, 0, 0b010],
        '-' => [0, 0, 0b111, 0, 0],
        '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
        _ => [0, 0, 0, 0, 0],
    }
}

fn fill(image: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: egui::Color32) {
    let max_x = x.saturating_add(w).min(image.width());
    let max_y = y.saturating_add(h).min(image.height());
    for row in y..max_y {
        for column in x..max_x {
            image.put_pixel(column, row, px(color));
        }
    }
}

fn stroke(image: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: egui::Color32) {
    fill(image, x, y, w, 1, color);
    fill(image, x, y.saturating_add(h).saturating_sub(1), w, 1, color);
    fill(image, x, y, 1, h, color);
    fill(image, x.saturating_add(w).saturating_sub(1), y, 1, h, color);
}

fn px(color: egui::Color32) -> Rgba<u8> {
    Rgba([color.r(), color.g(), color.b(), color.a()])
}
