// src/main.rs

fn main() {
    let native_options = eframe::NativeOptions::default();
    if let Err(e) = eframe::run_native(
        "BTC Black Hole Brute-Force",
        native_options,
        Box::new(|_cc| Box::new(btc_black_hole_rust::BruteForceApp::default())),
    ) {
        eprintln!("Error running application: {}", e);
    }
}