// Numera - A high-precision scientific calculator written in Rust.
// Licensed under GPL-2.0-or-later.
//
// Thin binary entry point. All logic lives in the library crate (lib.rs).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> eframe::Result<()> {
    let icon_bytes = include_bytes!("resources/logo.png");
    let img = image::load_from_memory(icon_bytes).unwrap().into_rgba8();
    let (w, h) = img.dimensions();
    let icon = egui::IconData {
        rgba: img.into_raw(),
        width: w,
        height: h,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([480.0, 320.0])
            .with_icon(std::sync::Arc::new(icon)),
        ..Default::default()
    };

    eframe::run_native(
        "Numera",
        options,
        Box::new(|cc| Ok(Box::new(numera::gui::NumeraApp::new(cc)))),
    )
}

