// Numera Side Panels
// History, Constants, Functions, Variables — polished collapsible panels.

use crate::constants;
use crate::functions;
use crate::math::{HNumber, NumberFormat};
use super::theme::Theme;

/// Constants panel
pub fn show_constants_panel(
    ui: &mut egui::Ui,
    filter: &mut String,
    insert_text: &mut Option<String>,
) {
    ui.collapsing(
        egui::RichText::new("🔬 Constants").strong().color(Theme::TEXT),
        |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(Theme::TEXT_DIM, "🔍");
                ui.add(
                    egui::TextEdit::singleline(filter)
                        .desired_width(ui.available_width())
                        .hint_text("Search…"),
                );
            });
            ui.add_space(2.0);

            let filter_lower = filter.to_lowercase();

            for category in constants::categories() {
                let consts: Vec<_> = if filter_lower.is_empty() {
                    constants::constants_in_category(category)
                } else {
                    constants::constants_in_category(category)
                        .into_iter()
                        .filter(|c| c.name.to_lowercase().contains(&filter_lower))
                        .collect()
                };

                if !consts.is_empty() {
                    ui.collapsing(category, |ui| {
                        for c in &consts {
                            ui.horizontal(|ui| {
                                if ui.small_button("⊕").on_hover_text("Insert value").clicked() {
                                    *insert_text = Some(c.value.to_string());
                                }
                                let unit_str = if c.unit.is_empty() {
                                    String::new()
                                } else {
                                    format!(" [{}]", c.unit)
                                };
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}: {}{}",
                                        c.name, c.value, unit_str
                                    ))
                                    .font(egui::FontId::proportional(11.0))
                                    .color(Theme::TEXT_DIM),
                                );
                            });
                        }
                    });
                }
            }
        },
    );
}

/// Functions panel
pub fn show_functions_panel(
    ui: &mut egui::Ui,
    filter: &mut String,
    insert_text: &mut Option<String>,
) {
    ui.collapsing(
        egui::RichText::new("ƒ Functions").strong().color(Theme::TEXT),
        |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(Theme::TEXT_DIM, "🔍");
                ui.add(
                    egui::TextEdit::singleline(filter)
                        .desired_width(ui.available_width())
                        .hint_text("Search…"),
                );
            });
            ui.add_space(2.0);

            let filter_lower = filter.to_lowercase();

            for category in functions::categories() {
                let funcs: Vec<_> = functions::all_functions()
                    .into_iter()
                    .filter(|f| {
                        f.category == category
                            && (filter_lower.is_empty()
                                || f.name.contains(&filter_lower)
                                || f.description.to_lowercase().contains(&filter_lower))
                    })
                    .collect();

                if !funcs.is_empty() {
                    ui.collapsing(category, |ui| {
                        for func in &funcs {
                            ui.horizontal(|ui| {
                                if ui
                                    .small_button("⊕")
                                    .on_hover_text("Insert function")
                                    .clicked()
                                {
                                    *insert_text = Some(format!("{}(", func.name));
                                }
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}() — {}",
                                        func.name, func.description
                                    ))
                                    .font(egui::FontId::proportional(11.0))
                                    .color(Theme::TEXT_DIM),
                                );
                            });
                        }
                    });
                }
            }
        },
    );
}

/// Variables panel
pub fn show_variables_panel(
    ui: &mut egui::Ui,
    variables: &[(String, HNumber)],
    filter: &mut String,
    insert_text: &mut Option<String>,
    format: NumberFormat,
    precision: i32,
    radix_char: char,
) {
    ui.collapsing(
        egui::RichText::new("📐 Variables").strong().color(Theme::TEXT),
        |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(Theme::TEXT_DIM, "🔍");
                ui.add(
                    egui::TextEdit::singleline(filter)
                        .desired_width(ui.available_width())
                        .hint_text("Search…"),
                );
            });
            ui.add_space(2.0);

            let filter_lower = filter.to_lowercase();

            if variables.is_empty() {
                ui.colored_label(Theme::TEXT_DIM, "No user variables defined.");
            } else {
                for (name, value) in variables {
                    if filter_lower.is_empty() || name.to_lowercase().contains(&filter_lower) {
                        let val_str = value.format_with(format, precision, radix_char);
                        ui.horizontal(|ui| {
                            if ui
                                .small_button("⊕")
                                .on_hover_text("Insert variable name")
                                .clicked()
                            {
                                *insert_text = Some(name.clone());
                            }
                            ui.label(
                                egui::RichText::new(format!("{} = {}", name, val_str))
                                    .font(egui::FontId::monospace(11.0))
                                    .color(Theme::TEXT_DIM),
                            );
                        });
                    }
                }
            }
        },
    );
}
