//! Visual Rust Studio — point d'entrée.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use vrs_app::VrsApp;

fn main() -> eframe::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let app = VrsApp::load();
    let title = app.i18n.tr("app-title");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title.clone())
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Visual Rust Studio",
        options,
        Box::new(|cc| {
            app.apply_style(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}
