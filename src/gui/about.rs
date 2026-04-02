// Numera About Dialog — with logo

use super::theme::Theme;

pub fn show_about(
    ctx: &egui::Context,
    open: &mut bool,
    logo_texture: &mut Option<egui::TextureHandle>,
) {
    let mut should_close = false;

    // Ensure logo is loaded
    let tex = if let Some(ref t) = logo_texture {
        t.clone()
    } else {
        let icon_bytes = include_bytes!("../resources/logo.png");
        let img = image::load_from_memory(icon_bytes).unwrap().into_rgba8();
        let (w, h) = img.dimensions();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize],
            img.as_raw(),
        );
        let t = ctx.load_texture("about_logo", color_image, egui::TextureOptions::LINEAR);
        *logo_texture = Some(t.clone());
        t
    };

    egui::Window::new("About Numera")
        .resizable(false)
        .collapsible(false)
        .default_size([420.0, 460.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);

                // Logo
                ui.add(
                    egui::Image::new(&tex)
                        .max_size(egui::vec2(80.0, 80.0))
                        .corner_radius(egui::CornerRadius::same(12)),
                );

                ui.add_space(8.0);
                ui.heading(
                    egui::RichText::new("Numera")
                        .size(28.0)
                        .strong()
                        .color(Theme::ACCENT),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new("High-Precision Scientific Calculator")
                        .size(13.0)
                        .color(Theme::TEXT_DIM),
                );
                ui.label(
                    egui::RichText::new("Written in Rust with egui")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(100, 100, 120)),
                );

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(4.0);

                ui.label(
                    egui::RichText::new("Based on SpeedCrunch:")
                        .strong()
                        .size(12.0),
                );
                ui.label(
                    egui::RichText::new("Original by Ariya Hidayat & Helder Correia")
                        .size(11.0)
                        .color(Theme::TEXT_DIM),
                );
                ui.label(
                    egui::RichText::new("Licensed under GPL-2.0-or-later")
                        .size(11.0)
                        .color(Theme::TEXT_DIM),
                );

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                ui.label(egui::RichText::new("Features:").strong().size(12.0));
                let features = [
                    "• Expression evaluator with operator precedence",
                    "• 67+ mathematical functions",
                    "• 45+ physical constants",
                    "• Variable support with assignment",
                    "• Multiple number formats (dec/hex/oct/bin)",
                    "• Implicit function syntax (sin pi)",
                    "• On-screen keypad · Session persistence",
                ];
                for feat in features {
                    ui.label(egui::RichText::new(feat).size(11.0).color(Theme::TEXT_DIM));
                }

                ui.add_space(10.0);
                if ui.button("Close").clicked() {
                    should_close = true;
                }
            });
        });

    if should_close {
        *open = false;
    }
}
