use std::sync::Arc;

use egui::{text::LayoutJob, Color32, FontId, TextFormat, TextStyle, Ui};

#[derive(Clone, Copy)]
struct Palette {
    plain: Color32,
    key: Color32,
    string: Color32,
    number: Color32,
    keyword: Color32,
    punct: Color32,
    tag: Color32,
    attr: Color32,
    comment: Color32,
    method: Color32,
    class: Color32,
}

pub(crate) fn layouter() -> impl FnMut(&Ui, &str, f32) -> Arc<egui::Galley> {
    move |ui, text, wrap_width| {
        let mut job = highlight_payload(ui, text);
        job.wrap.max_width = wrap_width;
        ui.fonts(|fonts| fonts.layout_job(job))
    }
}

pub(crate) fn javascript_layouter() -> impl FnMut(&Ui, &str, f32) -> Arc<egui::Galley> {
    move |ui, text, wrap_width| {
        let font = TextStyle::Monospace.resolve(ui.style());
        let mut job = highlight_javascript(text, font, palette(ui));
        job.wrap.max_width = wrap_width;
        ui.fonts(|fonts| fonts.layout_job(job))
    }
}

fn highlight_payload(ui: &Ui, text: &str) -> LayoutJob {
    let font = TextStyle::Monospace.resolve(ui.style());
    let palette = palette(ui);
    if looks_like_xml(text) {
        highlight_xml(text, font, palette)
    } else if looks_like_json(text) {
        highlight_json(text, font, palette)
    } else {
        plain_job(text, font, palette.plain)
    }
}

fn looks_like_json(text: &str) -> bool {
    matches!(text.trim_start().chars().next(), Some('{') | Some('['))
}

fn looks_like_xml(text: &str) -> bool {
    text.trim_start().starts_with('<')
}

fn palette(ui: &Ui) -> Palette {
    let visuals = ui.visuals();
    if visuals.dark_mode {
        Palette {
            plain: visuals.text_color(),
            key: Color32::from_rgb(156, 220, 254),
            string: Color32::from_rgb(206, 145, 120),
            number: Color32::from_rgb(181, 206, 168),
            keyword: Color32::from_rgb(86, 156, 214),
            punct: visuals.weak_text_color(),
            tag: Color32::from_rgb(86, 156, 214),
            attr: Color32::from_rgb(156, 220, 254),
            comment: Color32::from_rgb(106, 153, 85),
            method: Color32::from_rgb(220, 220, 170),
            class: Color32::from_rgb(78, 201, 176),
        }
    } else {
        Palette {
            plain: visuals.text_color(),
            key: Color32::from_rgb(0, 92, 160),
            string: Color32::from_rgb(163, 21, 21),
            number: Color32::from_rgb(9, 134, 88),
            keyword: Color32::from_rgb(0, 0, 255),
            punct: visuals.weak_text_color(),
            tag: Color32::from_rgb(128, 0, 0),
            attr: Color32::from_rgb(255, 0, 0),
            comment: Color32::from_rgb(0, 128, 0),
            method: Color32::from_rgb(121, 94, 38),
            class: Color32::from_rgb(38, 127, 153),
        }
    }
}

fn plain_job(text: &str, font: FontId, color: Color32) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, text, font, color);
    job
}

fn highlight_json(text: &str, font: FontId, palette: Palette) -> LayoutJob {
    let mut job = LayoutJob::default();
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < text.len() {
        let ch = text[index..].chars().next().unwrap_or_default();
        if ch == '"' {
            let end = string_end(text, index);
            let color = if next_non_ws(bytes, end) == Some(b':') {
                palette.key
            } else {
                palette.string
            };
            append(&mut job, &text[index..end], font.clone(), color);
            index = end;
        } else if ch.is_ascii_digit() || ch == '-' {
            let end = take_while(text, index, |c| {
                c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E')
            });
            append(&mut job, &text[index..end], font.clone(), palette.number);
            index = end;
        } else if starts_keyword(text, index, "true")
            || starts_keyword(text, index, "false")
            || starts_keyword(text, index, "null")
        {
            let end = take_while(text, index, |c| c.is_ascii_alphabetic());
            append(&mut job, &text[index..end], font.clone(), palette.keyword);
            index = end;
        } else if matches!(ch, '{' | '}' | '[' | ']' | ':' | ',') {
            append(
                &mut job,
                &text[index..index + ch.len_utf8()],
                font.clone(),
                palette.punct,
            );
            index += ch.len_utf8();
        } else {
            append(
                &mut job,
                &text[index..index + ch.len_utf8()],
                font.clone(),
                palette.plain,
            );
            index += ch.len_utf8();
        }
    }
    job
}

fn highlight_xml(text: &str, font: FontId, palette: Palette) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mut index = 0;
    while index < text.len() {
        if text[index..].starts_with("<!--") {
            let end = text[index..]
                .find("-->")
                .map_or(text.len(), |offset| index + offset + 3);
            append(&mut job, &text[index..end], font.clone(), palette.comment);
            index = end;
        } else if text[index..].starts_with('<') {
            let end = text[index..]
                .find('>')
                .map_or(text.len(), |offset| index + offset + 1);
            highlight_xml_tag(&mut job, &text[index..end], font.clone(), palette);
            index = end;
        } else {
            let end = text[index..]
                .find('<')
                .map_or(text.len(), |offset| index + offset);
            append(&mut job, &text[index..end], font.clone(), palette.plain);
            index = end;
        }
    }
    job
}

fn highlight_xml_tag(job: &mut LayoutJob, tag: &str, font: FontId, palette: Palette) {
    let mut index = 0;
    while index < tag.len() {
        let ch = tag[index..].chars().next().unwrap_or_default();
        if ch == '"' || ch == '\'' {
            let end = quoted_end(tag, index, ch);
            append(job, &tag[index..end], font.clone(), palette.string);
            index = end;
        } else if ch.is_ascii_alphabetic() || matches!(ch, '_' | ':' | '-') {
            let end = take_while(tag, index, |c| {
                c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '-' | '.')
            });
            let color = if previous_non_ws(tag.as_bytes(), index) == Some(b'<')
                || previous_non_ws(tag.as_bytes(), index) == Some(b'/')
            {
                palette.tag
            } else {
                palette.attr
            };
            append(job, &tag[index..end], font.clone(), color);
            index = end;
        } else if matches!(ch, '<' | '>' | '/' | '=') {
            append(
                job,
                &tag[index..index + ch.len_utf8()],
                font.clone(),
                palette.punct,
            );
            index += ch.len_utf8();
        } else {
            append(
                job,
                &tag[index..index + ch.len_utf8()],
                font.clone(),
                palette.plain,
            );
            index += ch.len_utf8();
        }
    }
}

fn highlight_javascript(text: &str, font: FontId, palette: Palette) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mut index = 0;
    while index < text.len() {
        let ch = text[index..].chars().next().unwrap_or_default();
        if text[index..].starts_with("//") {
            let end = text[index..]
                .find('\n')
                .map_or(text.len(), |offset| index + offset);
            append(&mut job, &text[index..end], font.clone(), palette.comment);
            index = end;
        } else if text[index..].starts_with("/*") {
            let end = text[index..]
                .find("*/")
                .map_or(text.len(), |offset| index + offset + 2);
            append(&mut job, &text[index..end], font.clone(), palette.comment);
            index = end;
        } else if matches!(ch, '"' | '\'' | '`') {
            let end = quoted_end_escaped(text, index, ch);
            append(&mut job, &text[index..end], font.clone(), palette.string);
            index = end;
        } else if ch.is_ascii_digit() {
            let end = take_while(text, index, |c| {
                c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '+' | '-')
            });
            append(&mut job, &text[index..end], font.clone(), palette.number);
            index = end;
        } else if is_js_ident_start(ch) {
            let end = take_while(text, index, is_js_ident_continue);
            let word = &text[index..end];
            let color = if is_js_keyword(word) {
                palette.keyword
            } else if is_class_like(word) || previous_word_is_new(text, index) {
                palette.class
            } else if previous_non_ws(text.as_bytes(), index) == Some(b'.')
                || next_non_ws(text.as_bytes(), end) == Some(b'(')
            {
                palette.method
            } else {
                palette.plain
            };
            append(&mut job, word, font.clone(), color);
            index = end;
        } else if is_js_punctuation(ch) {
            append(
                &mut job,
                &text[index..index + ch.len_utf8()],
                font.clone(),
                palette.punct,
            );
            index += ch.len_utf8();
        } else {
            append(
                &mut job,
                &text[index..index + ch.len_utf8()],
                font.clone(),
                palette.plain,
            );
            index += ch.len_utf8();
        }
    }
    job
}

fn append(job: &mut LayoutJob, text: &str, font: FontId, color: Color32) {
    job.append(text, 0.0, TextFormat::simple(font, color));
}

fn string_end(text: &str, start: usize) -> usize {
    let mut escaped = false;
    for (offset, ch) in text[start + 1..].char_indices() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return start + 1 + offset + ch.len_utf8();
        }
    }
    text.len()
}

fn quoted_end(text: &str, start: usize, quote: char) -> usize {
    for (offset, ch) in text[start + 1..].char_indices() {
        if ch == quote {
            return start + 1 + offset + ch.len_utf8();
        }
    }
    text.len()
}

fn quoted_end_escaped(text: &str, start: usize, quote: char) -> usize {
    let mut escaped = false;
    for (offset, ch) in text[start + 1..].char_indices() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            return start + 1 + offset + ch.len_utf8();
        }
    }
    text.len()
}

fn take_while(text: &str, start: usize, mut predicate: impl FnMut(char) -> bool) -> usize {
    for (offset, ch) in text[start..].char_indices() {
        if !predicate(ch) {
            return start + offset;
        }
    }
    text.len()
}

fn next_non_ws(bytes: &[u8], start: usize) -> Option<u8> {
    bytes
        .get(start..)?
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
}

fn previous_non_ws(bytes: &[u8], start: usize) -> Option<u8> {
    bytes
        .get(..start)?
        .iter()
        .rev()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
}

fn starts_keyword(text: &str, index: usize, keyword: &str) -> bool {
    text[index..].starts_with(keyword)
        && text[index + keyword.len()..]
            .chars()
            .next()
            .is_none_or(|ch| !ch.is_ascii_alphabetic())
}

fn is_js_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, '_' | '$')
}

fn is_js_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$')
}

fn is_js_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '{' | '}'
            | '['
            | ']'
            | '('
            | ')'
            | ';'
            | ','
            | '.'
            | ':'
            | '?'
            | '!'
            | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '='
            | '<'
            | '>'
            | '&'
            | '|'
    )
}

fn is_js_keyword(word: &str) -> bool {
    matches!(
        word,
        "async"
            | "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "from"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "let"
            | "new"
            | "null"
            | "of"
            | "return"
            | "static"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "undefined"
            | "var"
            | "void"
            | "while"
            | "yield"
    )
}

fn is_class_like(word: &str) -> bool {
    word.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
}

fn previous_word_is_new(text: &str, index: usize) -> bool {
    let Some(prefix) = text.get(..index) else {
        return false;
    };
    let trimmed = prefix.trim_end();
    let Some(end) = trimmed
        .char_indices()
        .last()
        .map(|(index, ch)| index + ch.len_utf8())
    else {
        return false;
    };
    let start = trimmed[..end]
        .char_indices()
        .rev()
        .find(|(_, ch)| !is_js_ident_continue(*ch))
        .map_or(0, |(index, ch)| index + ch.len_utf8());
    &trimmed[start..end] == "new"
}
