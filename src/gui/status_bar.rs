// Numera Status Bar
// Bottom bar showing current angle mode, format, precision, and live preview.

use crate::settings::Settings;
use super::theme::Theme;

pub fn show_status_bar(ctx: &egui::Context, settings: &Settings, preview_text: &str) {
    if !settings.show_status_bar {
        return;
    }

    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(22.0)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 16.0;

                let pill = |ui: &mut egui::Ui, label: &str| {
                    ui.label(
                        egui::RichText::new(label)
                            .size(10.5)
                            .color(Theme::TEXT_DIM),
                    );
                };

                pill(ui, settings.angle_mode_label());
                pill(ui, settings.format_label());
                pill(ui, &settings.precision_label());

                if !preview_text.is_empty() {
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.label(
                                egui::RichText::new(preview_text)
                                    .font(egui::FontId::monospace(11.0))
                                    .color(Theme::TEXT_DIM),
                            );
                            ui.label(
                                egui::RichText::new("≈")
                                    .size(11.0)
                                    .color(Theme::ACCENT),
                            );
                        },
                    );
                }
            });
        });
}
