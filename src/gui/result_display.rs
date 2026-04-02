// Numera Result Display
// Renders the scrollable history of expressions and results.

use crate::history::HistoryEntry;
use crate::math::NumberFormat;
use super::theme::Theme;

pub fn show_results(
    ui: &mut egui::Ui,
    history: &[HistoryEntry],
    format: NumberFormat,
    precision: i32,
    radix_char: char,
    editor_text: &mut String,
) {
    if history.is_empty() {
        show_empty_placeholder(ui);
        return;
    }

    for (index, entry) in history.iter().enumerate() {
        show_history_row(ui, entry, index, format, precision, radix_char, editor_text);
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
            show_expression_line(ui, entry, editor_text);
            show_result_line(ui, entry, format, precision, radix_char, editor_text);
        });
}

fn show_expression_line(
    ui: &mut egui::Ui,
    entry: &HistoryEntry,
    editor_text: &mut String,
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        let label = egui::RichText::new(&entry.expression)
            .font(egui::FontId::monospace(12.0))
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
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        let color = if entry.is_error { Theme::ERROR } else { Theme::TEXT_RESULT };

        let display_text = if entry.is_error {
            entry.result.clone()
        } else {
            let entry_format = entry.format_override.unwrap_or(format);
            entry.value.format_with(entry_format, precision, radix_char)
        };

        let label = egui::RichText::new(&display_text)
            .font(egui::FontId::monospace(15.0))
            .strong()
            .color(color);

        let response = ui.add(egui::Label::new(label).selectable(true));
        if response.clicked() {
            ui.ctx().copy_text(display_text.clone());
        }
        if response.double_clicked() {
            *editor_text = display_text;
        }
        response.on_hover_text("Click to copy · Double-click to insert · Drag to select");
    });
}
