//! SelfPaint — a Windows 11 Paint clone for Linux, built on eframe/egui + craft-ui.

mod app;
mod canvas;
mod document;
mod fonts;
mod history;
mod i18n;
mod palette;
mod project;
mod ribbon;
mod settings;
mod tools;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("SelfPaint")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon-256.png"))
                    .expect("valid icon PNG"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "SelfPaint",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let mut visuals = egui::Visuals::light();
            palette::apply_theme(&mut visuals);
            cc.egui_ctx.set_visuals(visuals);
            Ok(Box::new(app::SelfPaintApp::new()))
        }),
    )
}
