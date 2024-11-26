// src/main.rs

use btc_black_hole_rust::BruteForceApp;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "BTC Black Hole",
        options,
        Box::new(|_cc| {
            // Wrap the Box in Ok to match the expected Result type
            Ok(Box::new(BruteForceApp::default()))
        }),
    )
    .expect("Failed to run the eframe application");
}