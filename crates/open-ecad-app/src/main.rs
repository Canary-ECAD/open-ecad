//! open-ecad: prototipo de la ventana principal (MAPA-UI.md §4).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod canvas;
mod commands;
mod mock;
mod panels;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("open-ecad")
            .with_inner_size([1400.0, 860.0])
            .with_maximized(true)
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native("open-ecad", options, Box::new(|cc| Ok(Box::new(app::OpenEcadApp::new(cc)))))
}
