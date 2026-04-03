// Numera Menu Bar
// Top-level application menu: Session, View, Settings, Help.

use crate::math::{AngleMode, NumberFormat};
use crate::settings::{Settings, FontFamily, FontSettings};
use crate::evaluator::Evaluator;

#[allow(clippy::too_many_arguments)]
pub fn show_menu_bar(
    ctx: &egui::Context,
    settings: &mut Settings,
    evaluator: &mut Evaluator,
    _history_len: usize,
    clear_history: &mut bool,
    clear_variables: &mut bool,
    show_about: &mut bool,
    quit: &mut bool,
) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            show_session_menu(ui, clear_history, clear_variables, quit);
            show_view_menu(ui, settings);
            show_settings_menu(ui, settings, evaluator);
            show_help_menu(ui, show_about);
        });
    });
}

fn show_session_menu(
    ui: &mut egui::Ui,
    clear_history: &mut bool,
    clear_variables: &mut bool,
    quit: &mut bool,
) {
    ui.menu_button("Session", |ui| {
        if ui.button("Clear History").clicked() {
            *clear_history = true;
            ui.close_menu();
        }
        if ui.button("Clear Variables").clicked() {
            *clear_variables = true;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Save & Quit").clicked() {
            *quit = true;
            ui.close_menu();
        }
    });
}

fn show_view_menu(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.menu_button("View", |ui| {
        ui.checkbox(&mut settings.show_keypad, "Keypad");
        ui.checkbox(&mut settings.show_functions, "Functions Panel");
        ui.checkbox(&mut settings.show_constants, "Constants Panel");
        ui.checkbox(&mut settings.show_variables, "Variables Panel");
        ui.checkbox(&mut settings.show_status_bar, "Status Bar");
    });
}

fn show_settings_menu(
    ui: &mut egui::Ui,
    settings: &mut Settings,
    evaluator: &mut Evaluator,
) {
    ui.menu_button("Settings", |ui| {
        show_angle_submenu(ui, settings, evaluator);
        show_format_submenu(ui, settings);
        show_precision_submenu(ui, settings);
        show_radix_submenu(ui, settings);
        ui.separator();
        show_font_submenu(ui, settings);
        ui.separator();
        ui.checkbox(&mut settings.auto_calc, "Live Preview");
        ui.checkbox(&mut settings.save_session, "Save Session");
        ui.checkbox(&mut settings.save_variables, "Save Variables");
    });
}

fn show_angle_submenu(
    ui: &mut egui::Ui,
    settings: &mut Settings,
    evaluator: &mut Evaluator,
) {
    ui.menu_button(
        format!("Angle: {}", settings.angle_mode_label()),
        |ui| {
            for (mode, label) in [(AngleMode::Radian, "Radian"), (AngleMode::Degree, "Degree")] {
                if ui.radio(evaluator.angle_mode == mode, label).clicked() {
                    evaluator.angle_mode = mode;
                    settings.angle_mode = mode;
                    ui.close_menu();
                }
            }
        },
    );
}

fn show_format_submenu(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.menu_button(
        format!("Format: {}", settings.format_label()),
        |ui| {
            let formats = [
                (NumberFormat::General, "General"),
                (NumberFormat::Fixed, "Fixed"),
                (NumberFormat::Scientific, "Scientific"),
                (NumberFormat::Engineering, "Engineering"),
                (NumberFormat::Hexadecimal, "Hexadecimal"),
                (NumberFormat::Octal, "Octal"),
                (NumberFormat::Binary, "Binary"),
            ];
            for (format, label) in formats {
                if ui.radio(settings.result_format == format, label).clicked() {
                    settings.result_format = format;
                    ui.close_menu();
                }
            }
        },
    );
}

fn show_precision_submenu(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.menu_button(
        format!("Precision: {}", settings.precision_label()),
        |ui| {
            if ui.radio(settings.precision == -1, "Auto").clicked() {
                settings.precision = -1;
                ui.close_menu();
            }
            for digits in [2, 4, 8, 15, 30, 50] {
                if ui.radio(settings.precision == digits, format!("{} digits", digits)).clicked() {
                    settings.precision = digits;
                    ui.close_menu();
                }
            }
        },
    );
}

fn show_radix_submenu(ui: &mut egui::Ui, settings: &mut Settings) {
    let current_label = if settings.radix_char == '.' { "Dot (.)" } else { "Comma (,)" };
    ui.menu_button(format!("Radix: {}", current_label), |ui| {
        for (character, label) in [('.', "Dot (.)"), (',', "Comma (,)")] {
            if ui.radio(settings.radix_char == character, label).clicked() {
                settings.radix_char = character;
                ui.close_menu();
            }
        }
    });
}

fn show_font_submenu(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.menu_button("Fonts", |ui| {
        show_font_settings(ui, "Expression", &mut settings.expression_font);
        show_font_settings(ui, "Result", &mut settings.result_font);
        show_font_settings(ui, "Input", &mut settings.input_font);
        ui.separator();
        if ui.button("Reset to Defaults").clicked() {
            settings.reset_fonts();
            ui.close_menu();
        }
    });
}

fn show_font_settings(ui: &mut egui::Ui, label: &str, font: &mut FontSettings) {
    ui.menu_button(
        format!("{}: {} {:.0}px", label, font.family.label(), font.size),
        |ui| {
            ui.label(egui::RichText::new("Family").strong());
            for family in [FontFamily::Monospace, FontFamily::Proportional] {
                if ui.radio(font.family == family, family.label()).clicked() {
                    font.family = family;
                }
            }
            ui.separator();
            ui.label(egui::RichText::new("Size").strong());
            ui.add(egui::Slider::new(&mut font.size, 8.0..=32.0).suffix("px"));
        },
    );
}

fn show_help_menu(ui: &mut egui::Ui, show_about: &mut bool) {
    ui.menu_button("Help", |ui| {
        if ui.button("About Numera…").clicked() {
            *show_about = true;
            ui.close_menu();
        }
    });
}
