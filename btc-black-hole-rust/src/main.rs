// src/main.rs

use btc_black_hole_rust::BruteForceApp;
use eframe::NativeOptions;
use eframe::egui::Vec2;

fn main() {
    let app = BruteForceApp::default();
    let native_options = NativeOptions {
        initial_window_size: Some(Vec2::new(1200.0, 800.0)),
        ..Default::default()
    };

    eframe::run_native(
        "BTC Black Hole Brute Force",
        native_options,
        Box::new(|_cc| Box::new(app)),
    )
    .unwrap();
}