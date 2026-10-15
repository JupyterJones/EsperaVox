mod camera;
mod viewport;
mod app;

use app::EsperaVoxApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("EsperaVox - Native Linux Voxel Studio")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "EsperaVox",
        native_options,
        Box::new(|cc| {
            // Apply a sleek, dark MagicaVoxel-inspired visual theme
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = egui::Color32::from_rgb(20, 24, 30);
            visuals.window_fill = egui::Color32::from_rgb(26, 32, 40);
            visuals.selection.bg_fill = egui::Color32::from_rgb(60, 140, 255);
            cc.egui_ctx.set_visuals(visuals);

            Ok(Box::new(EsperaVoxApp::default()))
        }),
    )
}
