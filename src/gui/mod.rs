// Numera GUI
// Application shell wiring evaluator, settings, and UI components.
// Each concern lives in its own module; this file is orchestration only.

pub mod about;
pub mod editor;
pub mod keypad;
pub mod menu_bar;
pub mod panels;
pub mod result_display;
pub mod status_bar;
pub mod theme;

use crate::evaluator::Evaluator;
use crate::history::HistoryEntry;
use crate::math::{HNumber, NumberFormat};
use crate::settings::Settings;

use editor::Editor;
use keypad::{KeyAction, Keypad, KeypadKey};
use theme::{Theme, apply_theme};

// ─── Application state ──────────────────────────────────────────────

pub struct NumeraApp {
    pub evaluator: Evaluator,
    pub settings: Settings,
    pub editor: Editor,
    pub history: Vec<HistoryEntry>,
    pub preview_text: String,
    pub history_nav_index: Option<usize>,

    pub constants_filter: String,
    pub functions_filter: String,
    pub variables_filter: String,

    pub show_about: bool,
    pub logo_texture: Option<egui::TextureHandle>,
}

impl NumeraApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_theme(&cc.egui_ctx);

        let settings = Settings::load();
        let mut evaluator = Evaluator::new();
        evaluator.angle_mode = settings.angle_mode;

        if settings.save_variables {
            for (name, value_str) in &settings.variables {
                if let Ok(value) = HNumber::from_str_radix(value_str) {
                    let _ = evaluator.set_variable(name, value);
                }
            }
        }

        let history = restore_history(&settings);

        NumeraApp {
            evaluator,
            settings,
            editor: Editor::new(),
            history,
            preview_text: String::new(),
            history_nav_index: None,

            constants_filter: String::new(),
            functions_filter: String::new(),
            variables_filter: String::new(),

            show_about: false,
            logo_texture: None,
        }
    }

    pub fn evaluate_current(&mut self) {
        let input = self.editor.text.trim().to_string();
        if input.is_empty() {
            return;
        }

        let (variable_name, expression) = parse_assignment(&input);

        match self.evaluator.evaluate(&expression, self.settings.radix_char) {
            Ok((value, format_override)) => {
                let result = value.format_with(
                    format_override.unwrap_or(self.settings.result_format),
                    self.settings.precision,
                    self.settings.radix_char,
                );
                let _ = self.evaluator.set_variable("ans", value.clone());
                if let Some(ref name) = variable_name {
                    let _ = self.evaluator.set_variable(name, value.clone());
                }
                self.history.push(HistoryEntry::success(input, value, result, format_override));
            }
            Err(message) => {
                self.history.push(HistoryEntry::error(input, message));
            }
        }

        self.editor.text.clear();
        self.preview_text.clear();
        self.history_nav_index = None;
    }

    pub fn update_preview(&mut self) {
        let text = self.editor.text.trim();
        if text.is_empty() || !self.settings.auto_calc {
            self.preview_text.clear();
            return;
        }
        self.preview_text = match self.evaluator.evaluate(text, self.settings.radix_char) {
            Ok((value, format_override)) => value.format_with(
                format_override.unwrap_or(self.settings.result_format),
                self.settings.precision,
                self.settings.radix_char,
            ),
            Err(_) => String::new(),
        };
    }

    pub fn save_state(&mut self) {
        self.settings.history_expressions =
            self.history.iter().map(|e| e.expression.clone()).collect();
        self.settings.history_results =
            self.history.iter().map(|e| e.result.clone()).collect();

        self.settings.variables.clear();
        for (name, value) in self.evaluator.user_variables() {
            self.settings.variables.insert(
                name,
                value.format_with(NumberFormat::General, -1, '.'),
            );
        }

        self.settings.angle_mode = self.evaluator.angle_mode;
        self.settings.save();
    }

    pub fn apply_keypad_key(&mut self, key: KeypadKey) {
        match key.action(self.settings.radix_char) {
            KeyAction::Insert(text) => self.editor.text.push_str(text),
            KeyAction::InsertChar(ch) => self.editor.text.push(ch),
            KeyAction::InsertDynamic(text) => self.editor.text.push_str(&text),
            KeyAction::Clear => self.editor.text.clear(),
            KeyAction::Backspace => { self.editor.text.pop(); }
            KeyAction::Evaluate => { self.evaluate_current(); return; }
        }
        self.update_preview();
    }

    pub fn navigate_history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let index = match self.history_nav_index {
            None => self.history.len() - 1,
            Some(i) => i.saturating_sub(1),
        };
        self.history_nav_index = Some(index);
        self.editor.text = self.history[index].expression.clone();
    }

    pub fn navigate_history_down(&mut self) {
        if let Some(index) = self.history_nav_index {
            if index + 1 < self.history.len() {
                self.history_nav_index = Some(index + 1);
                self.editor.text = self.history[index + 1].expression.clone();
            } else {
                self.history_nav_index = None;
                self.editor.text.clear();
            }
        }
    }

    pub fn clear_variables(&mut self) {
        let names: Vec<_> = self.evaluator.user_variables()
            .iter()
            .map(|(n, _)| n.clone())
            .collect();
        for name in names {
            self.evaluator.delete_variable(&name);
        }
    }
}

// ─── eframe::App ─────────────────────────────────────────────────────

impl eframe::App for NumeraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut clear_history = false;
        let mut clear_variables = false;
        let mut quit = false;

        menu_bar::show_menu_bar(
            ctx,
            &mut self.settings,
            &mut self.evaluator,
            self.history.len(),
            &mut clear_history,
            &mut clear_variables,
            &mut self.show_about,
            &mut quit,
        );

        if clear_history { self.history.clear(); }
        if clear_variables { self.clear_variables(); }
        if quit { self.save_state(); std::process::exit(0); }

        status_bar::show_status_bar(ctx, &self.settings, &self.preview_text);
        self.show_side_panels(ctx);

        if self.show_about {
            about::show_about(ctx, &mut self.show_about, &mut self.logo_texture);
        }

        self.show_central_panel(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_state();
    }
}

// ─── Layout sections (private, thin wiring) ──────────────────────────

impl NumeraApp {
    fn show_side_panels(&mut self, ctx: &egui::Context) {
        let any_panel = self.settings.show_constants
            || self.settings.show_functions
            || self.settings.show_variables;

        if !any_panel {
            return;
        }

        egui::SidePanel::left("side_panel")
            .default_width(240.0)
            .min_width(180.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().drag_to_scroll(false).show(ui, |ui| {
                    let mut insert_text: Option<String> = None;

                    if self.settings.show_constants {
                        panels::show_constants_panel(ui, &mut self.constants_filter, &mut insert_text);
                        ui.add_space(4.0);
                    }
                    if self.settings.show_functions {
                        panels::show_functions_panel(ui, &mut self.functions_filter, &mut insert_text);
                        ui.add_space(4.0);
                    }
                    if self.settings.show_variables {
                        let variables = self.evaluator.user_variables();
                        if let Some(name) = panels::show_variables_panel(
                            ui, &variables, &mut self.variables_filter, &mut insert_text,
                            self.settings.result_format, self.settings.precision, self.settings.radix_char,
                        ) {
                            self.evaluator.delete_variable(&name);
                        }
                    }

                    if let Some(text) = insert_text {
                        self.editor.text.push_str(&text);
                    }
                });
            });
    }

    fn show_central_panel(&mut self, ctx: &egui::Context) {
        // Editor + keypad pinned to the bottom
        egui::TopBottomPanel::bottom("editor_panel")
            .show_separator_line(true)
            .show(ctx, |ui| {
                let input_font_id = match self.settings.input_font.family {
                    crate::settings::FontFamily::Monospace => egui::FontId::monospace(self.settings.input_font.size),
                    crate::settings::FontFamily::Proportional => egui::FontId::proportional(self.settings.input_font.size),
                };
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.editor.text)
                        .font(input_font_id)
                        .desired_width(f32::INFINITY)
                        .hint_text(egui::RichText::new("Type expression…").color(Theme::TEXT_DIM))
                        .text_color(Theme::TEXT)
                        .frame(true)
                        .margin(egui::Margin::symmetric(8, 6)),
                );

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.evaluate_current();
                    response.request_focus();
                } else if response.changed() {
                    self.update_preview();
                }

                if response.has_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        self.navigate_history_up();
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        self.navigate_history_down();
                    }
                }

                if self.history.is_empty() && self.editor.text.is_empty() {
                    response.request_focus();
                }

                if self.settings.show_keypad {
                    ui.add_space(4.0);
                    let mut key_pressed = None;
                    Keypad::show(ui, self.settings.radix_char, &mut key_pressed);
                    if let Some(key) = key_pressed {
                        self.apply_keypad_key(key);
                    }
                }
            });

        // History fills remaining space, sticks to bottom like a terminal
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    // Push history to the bottom by adding flexible space above
                    let content_height = self.history.len() as f32 * 52.0; // approximate row height
                    let available = ui.available_height();
                    if content_height < available {
                        ui.add_space(available - content_height);
                    }

                    result_display::show_results(
                        ui, &self.history,
                        self.settings.result_format, self.settings.precision,
                        self.settings.radix_char, &mut self.editor.text,
                        self.settings.expression_font, self.settings.result_font,
                    );
                });
        });
    }
}

// ─── Pure helpers ────────────────────────────────────────────────────

fn parse_assignment(input: &str) -> (Option<String>, String) {
    if let Some(pos) = input.find('=') {
        let lhs = input[..pos].trim();
        let rhs = input[pos + 1..].trim();
        if !lhs.is_empty()
            && !rhs.is_empty()
            && lhs.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            return (Some(lhs.to_string()), rhs.to_string());
        }
    }
    (None, input.to_string())
}

fn restore_history(settings: &Settings) -> Vec<HistoryEntry> {
    if !settings.save_session {
        return Vec::new();
    }

    let count = settings.history_expressions.len().min(settings.history_results.len());
    let mut history = Vec::with_capacity(count);
    let mut temp_evaluator = Evaluator::new();
    temp_evaluator.angle_mode = settings.angle_mode;

    for i in 0..count {
        let expression = settings.history_expressions[i].clone();
        let result = settings.history_results[i].clone();
        let (value, format_override, is_error) = match temp_evaluator.evaluate(&expression, settings.radix_char) {
            Ok((v, fmt)) => (v, fmt, false),
            Err(_) => (HNumber::nan(), None, true),
        };
        history.push(HistoryEntry { expression, result, value, format_override, is_error });
    }

    history
}
