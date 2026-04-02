// Numera Theme
// Tokyo-Night inspired color palette and egui style configuration.
// Single source of truth for all visual constants.

/// Color palette used across all GUI components.
pub struct Theme;

impl Theme {
    pub const BG: egui::Color32 = egui::Color32::from_rgb(26, 27, 38);
    pub const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(30, 31, 44);
    pub const BG_EDITOR: egui::Color32 = egui::Color32::from_rgb(22, 23, 34);
    pub const BG_ROW_ALT: egui::Color32 = egui::Color32::from_rgb(32, 33, 48);
    pub const BG_BUTTON: egui::Color32 = egui::Color32::from_rgb(40, 42, 58);
    pub const BG_BUTTON_HOVER: egui::Color32 = egui::Color32::from_rgb(50, 52, 70);

    pub const TEXT: egui::Color32 = egui::Color32::from_rgb(192, 202, 224);
    pub const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(110, 118, 148);
    pub const TEXT_RESULT: egui::Color32 = egui::Color32::from_rgb(125, 207, 255);

    pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 120);
    pub const ACCENT_ORANGE: egui::Color32 = egui::Color32::from_rgb(255, 158, 100);
    pub const ERROR: egui::Color32 = egui::Color32::from_rgb(247, 118, 142);
    pub const BORDER: egui::Color32 = egui::Color32::from_rgb(50, 52, 70);
}

/// Apply the Numera dark theme to an egui context.
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    let visuals = &mut style.visuals;
    visuals.dark_mode = true;
    visuals.override_text_color = Some(Theme::TEXT);
    visuals.panel_fill = Theme::BG;
    visuals.window_fill = Theme::BG_PANEL;
    visuals.faint_bg_color = Theme::BG_ROW_ALT;
    visuals.extreme_bg_color = Theme::BG_EDITOR;
    visuals.window_stroke = egui::Stroke::new(1.0, Theme::BORDER);

    visuals.widgets.noninteractive.bg_fill = Theme::BG_PANEL;
    visuals.widgets.inactive.bg_fill = Theme::BG_BUTTON;
    visuals.widgets.hovered.bg_fill = Theme::BG_BUTTON_HOVER;
    visuals.widgets.active.bg_fill = Theme::ACCENT;

    visuals.selection.bg_fill = egui::Color32::from_rgba_premultiplied(80, 200, 120, 40);
    visuals.selection.stroke = egui::Stroke::new(1.0, Theme::ACCENT);

    style.spacing.item_spacing = egui::vec2(6.0, 4.0);

    ctx.set_style(style);
}
