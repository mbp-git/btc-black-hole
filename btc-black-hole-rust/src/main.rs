// src/main.rs

use btc_black_hole_rust::BruteForceApp;
use eframe::egui::Vec2;

fn main() {
    let app = BruteForceApp::default();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(900.0, 750.0)),
        ..Default::default()
    };
    eframe::run_native(
        "BTC Black Hole Rust",
        native_options,
        Box::new(|_cc| Box::new(app)),
    )
    .unwrap();
}