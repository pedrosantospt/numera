// Numera Result Display
// Renders the scrollable history of expressions and results.

use crate::history::HistoryEntry;
use crate::math::NumberFormat;
use crate::settings::{FontFamily, FontSettings};
use super::theme::Theme;

fn font_id(fs: FontSettings) -> egui::FontId {
    match fs.family {
        FontFamily::Monospace => egui::FontId::monospace(fs.size),
        FontFamily::Proportional => egui::FontId::proportional(fs.size),
    }
}

pub fn show_results(
    ui: &mut egui::Ui,
    history: &[HistoryEntry],
    format: NumberFormat,
    precision: i32,
    radix_char: char,
    editor_text: &mut String,
    expression_font: FontSettings,
    result_font: FontSettings,
) {
    if history.is_empty() {
        show_empty_placeholder(ui);
        return;
    }

    for (index, entry) in history.iter().enumerate() {
        show_history_row(ui, entry, index, format, precision, radix_char, editor_text, expression_font, result_font);
    }
}

fn show_empty_placeholder(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 3.0);
        ui.label(
            egui::RichText::new("Numera")
                .size(32.0)
                .color(egui::Color32::from_rgba_premultiplied(80, 200, 120, 60))
                .strong(),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Type an expression and press Enter")
                .size(14.0)
                .color(egui::Color32::from_rgb(80, 80, 100)),
        );
    });
}

fn show_history_row(
    ui: &mut egui::Ui,
    entry: &HistoryEntry,
    index: usize,
    format: NumberFormat,
    precision: i32,
    radix_char: char,
    editor_text: &mut String,
    expression_font: FontSettings,
    result_font: FontSettings,
) {
    let background = if index % 2 == 1 {
        Theme::BG_ROW_ALT
    } else {
        egui::Color32::TRANSPARENT
    };

    egui::Frame::new()
        .fill(background)
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(8, 5))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            show_expression_line(ui, entry, editor_text, expression_font);
            show_result_line(ui, entry, format, precision, radix_char, editor_text, result_font);
        });
}

fn show_expression_line(
    ui: &mut egui::Ui,
    entry: &HistoryEntry,
    editor_text: &mut String,
    ef: FontSettings,
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        let label = egui::RichText::new(&entry.expression)
            .font(font_id(ef))
            .color(Theme::TEXT_DIM);
        let response = ui.add(egui::Label::new(label).selectable(true));
        if response.clicked() {
            *editor_text = entry.expression.clone();
        }
        response.on_hover_text("Click to edit · Drag to select");
    });
}

fn show_result_line(
    ui: &mut egui::Ui,
    entry: &HistoryEntry,
    format: NumberFormat,
    precision: i32,
    radix_char: char,
    editor_text: &mut String,
    rf: FontSettings,
) {
    let color = if entry.is_error { Theme::ERROR } else { Theme::TEXT_RESULT };

    let display_text = if entry.is_error {
        entry.result.clone()
    } else {
        let entry_format = entry.format_override.unwrap_or(format);
        entry.value.format_with(entry_format, precision, radix_char)
    };

    let char_width = rf.size * 0.6;
    let max_chars = (ui.available_width() / char_width).max(10.0) as usize;
    let fits_one_line = display_text.len() <= max_chars;

    // For long results, insert newlines so digits break at consistent column widths.
    let wrapped_text = if fits_one_line {
        display_text.clone()
    } else {
        let mut result = String::with_capacity(display_text.len() + 10);
        let mut remaining = display_text.as_str();
        while remaining.len() > max_chars {
            result.push_str(&remaining[..max_chars]);
            result.push('\n');
            remaining = &remaining[max_chars..];
        }
        result.push_str(remaining);
        result
    };

    let label = egui::RichText::new(&wrapped_text)
        .font(font_id(rf))
        .strong()
        .color(color);

    let response = if fits_one_line {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            ui.add(egui::Label::new(label).selectable(true))
        }).inner
    } else {
        ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
            ui.add(egui::Label::new(label).selectable(true))
        }).inner
    };

    if response.clicked() {
        ui.ctx().copy_text(display_text.clone());
    }
    if response.double_clicked() {
        *editor_text = display_text;
    }
    response.on_hover_text("Click to copy · Double-click to insert · Drag to select");
}
