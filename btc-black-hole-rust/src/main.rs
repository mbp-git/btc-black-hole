// src/main.rs

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Instant;

use crossbeam::channel::{bounded, Receiver, Sender};
use eframe::egui::{CentralPanel, TextEdit, ProgressBar, Vec2};
use num_cpus;
use bitcoin::Address;
use std::str::FromStr;
// use btc_black_hole_rust::BruteForceApp;

/// Messages sent from brute-force threads to the GUI
enum Message {
    ProgressUpdate {
        thread_id: usize,
        progress: f32,
        time_remaining: String,
        hashes_per_second: String,
        current_candidate: String,
    },
    Found {
        candidate: String,
    },
    Finished,
    Cancelled,
    Error(String),
}

/// Structure to hold the application state.
struct BruteForceApp {
    base58_input: String,
    start_suffix: String,
    thread_count: usize,
    progress: f32,
    time_remaining: String,
    hashes_per_second: String,
    result: String,
    running: bool,
    stop_flag: Arc<AtomicBool>,
    receiver: Option<Receiver<Message>>,
    current_candidates: HashMap<usize, String>,
}

impl Default for BruteForceApp {
    fn default() -> Self {
        Self {
            base58_input: String::from("1BitcoinEaterAddressDontSendf59kuE"),
            start_suffix: String::from("1"),
            thread_count: num_cpus::get(),
            progress: 0.0,
            time_remaining: String::from("Calculating..."),
            hashes_per_second: String::from("Calculating..."),
            result: String::new(),
            running: false,
            stop_flag: Arc::new(AtomicBool::new(false)),
            receiver: None,
            current_candidates: HashMap::new(),
        }
    }
}

impl eframe::App for BruteForceApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            // Set background color
            ui.style_mut().visuals.window_fill = egui::Color32::from_rgb(30, 30, 30);
            ui.set_min_size(Vec2::new(800.0, 720.0));

            // Input Fields
            ui.horizontal_wrapped(|ui| {
                ui.label("Enter Base58 Address (without checksum):");
                ui.add(TextEdit::singleline(&mut self.base58_input).desired_width(400.0));
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("Starting Suffix (Base58):");
                ui.add(TextEdit::singleline(&mut self.start_suffix).desired_width(200.0));
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("Number of Threads:");
                ui.add(
                    egui::DragValue::new(&mut self.thread_count)
                        .clamp_range(1..=num_cpus::get())
                        .speed(1),
                );
                ui.label(format!("(Available CPUs: {})", num_cpus::get()));
            });

            ui.separator();

            // Current Candidate Address per Thread
            ui.label("Current Candidate Addresses per Thread:");
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                egui::Grid::new("thread_candidates_grid")
                    .striped(true)
                    .min_col_width(100.0)
                    .show(ui, |ui| {
                        ui.heading("Thread ID");
                        ui.heading("Current Candidate Address");
                        ui.end_row();

                        for (thread_id, candidate) in &self.current_candidates {
                            ui.label(format!("{}", thread_id));
                            ui.label(candidate);
                            ui.end_row();
                        }
                    });
            });

            // Progress Bar
            ui.add(ProgressBar::new(self.progress / 100.0).show_percentage());

            // Progress Details
            ui.horizontal(|ui| {
                ui.label(format!("Progress: {:.2}%", self.progress));
                ui.label(format!("Time Remaining: {}", self.time_remaining));
                ui.label(format!("Hashes per Second: {}", self.hashes_per_second));
            });

            ui.separator();

            // Result Display
            ui.label("Result:");
            ui.add(
                TextEdit::multiline(&mut self.result)
                    .desired_rows(10)
                    .desired_width(f32::INFINITY)
                    .lock_focus(true)
                    .min_size(Vec2::new(0.0, 150.0)),
            );

            ui.separator();

            // Start/Cancel Buttons
            if !self.running {
                if ui.button("Start Brute-Force").clicked() {
                    self.start_bruteforce();
                }
            } else {
                if ui.button("Cancel").clicked() {
                    self.stop_flag.store(true, Ordering::SeqCst);
                }
            }

            // Check for new messages
            if let Some(receiver) = &self.receiver {
                let mut messages = Vec::new();
                while let Ok(message) = receiver.try_recv() {
                    messages.push(message);
                }

                for message in messages {
                    match message {
                        Message::ProgressUpdate {
                            thread_id,
                            progress,
                            time_remaining,
                            hashes_per_second,
                            current_candidate,
                        } => {
                            self.progress = progress;
                            self.time_remaining = time_remaining;
                            self.hashes_per_second = hashes_per_second;
                            self.current_candidates.insert(thread_id, current_candidate);
                        }
                        Message::Found { candidate } => {
                            self.result = format!("Valid address found: {}", candidate);
                            self.progress = 100.0;
                            self.time_remaining = "Completed".to_string();
                            self.hashes_per_second = "N/A".to_string();
                            self.running = false;
                            self.receiver = None;
                            self.current_candidates.clear();
                        }
                        Message::Finished => {
                            self.result = "No valid address found.".to_string();
                            self.progress = 100.0;
                            self.time_remaining = "N/A".to_string();
                            self.hashes_per_second = "N/A".to_string();
                            self.running = false;
                            self.receiver = None;
                            self.current_candidates.clear();
                        }
                        Message::Cancelled => {
                            self.result = "Brute-forcing cancelled.".to_string();
                            self.progress = 0.0;
                            self.time_remaining = "N/A".to_string();
                            self.hashes_per_second = "N/A".to_string();
                            self.running = false;
                            self.receiver = None;
                            self.current_candidates.clear();
                        }
                        Message::Error(err) => {
                            self.result = format!("Error: {}", err);
                            self.running = false;
                            self.receiver = None;
                            self.current_candidates.clear();
                        }
                    }
                }
            }

            if self.running {
                ctx.request_repaint();
            }
        }); // Closing `CentralPanel`
    }
}

impl BruteForceApp {
    fn start_bruteforce(&mut self) {
        self.stop_flag.store(false, Ordering::SeqCst);

        let base58_input = self.base58_input.clone();
        let start_suffix = self.start_suffix.clone();
        let thread_count = self.thread_count;
        let stop_flag = Arc::clone(&self.stop_flag);
        self.running = true;
        self.result.clear();
        self.progress = 0.0;
        self.time_remaining = "Calculating...".to_string();
        self.hashes_per_second = "Calculating...".to_string();
        self.current_candidates.clear();

        let (tx, rx): (Sender<Message>, Receiver<Message>) = bounded(100);
        self.receiver = Some(rx);

        for thread_id in 0..thread_count {
            let base58_input = base58_input.clone();
            let start_suffix = start_suffix.clone();
            let tx = tx.clone();
            let stop_flag = Arc::clone(&stop_flag);

            thread::spawn(move || {
                brute_force_checksum(
                    base58_input,
                    start_suffix,
                    thread_id,
                    tx,
                    stop_flag,
                )
            });
        }
    }
}

fn brute_force_checksum(
    base58_input: String,
    start_suffix: String,
    thread_id: usize,
    tx: Sender<Message>,
    stop_flag: Arc<AtomicBool>,
) {
    let start_time = Instant::now();
    let base58_alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let base58_len = base58_alphabet.len() as u128;

    let current_length = base58_input.len();
    if current_length >= 34 {
        let _ = tx.send(Message::Error(
            "Input address already 34 characters or longer.".to_string(),
        ));
        return;
    }

    let chars_to_add = 34 - current_length;

    let mut start_index = 0u128;
    for (idx, char) in start_suffix.chars().rev().enumerate() {
        if let Some(pos) = base58_alphabet.find(char) {
            let base58_len_pow = base58_len.pow(idx as u32);
            start_index += pos as u128 * base58_len_pow;
        } else {
            let _ = tx.send(Message::Error(
                "Invalid character in starting suffix.".to_string(),
            ));
            return;
        }
    }

    let total_combinations = base58_len.pow(chars_to_add as u32);
    let mut combinations_checked = 0u128;

    for i in start_index..total_combinations {
        if stop_flag.load(Ordering::SeqCst) {
            let _ = tx.send(Message::Cancelled);
            return;
        }

        let mut temp = i;
        let mut suffix = Vec::with_capacity(chars_to_add);
        for _ in 0..chars_to_add {
            let idx = (temp % base58_len) as usize;
            suffix.push(base58_alphabet.chars().nth(idx).unwrap());
            temp /= base58_len;
        }
        suffix.reverse();
        let suffix_str: String = suffix.into_iter().collect();

        let candidate_address = format!("{}{}", base58_input, suffix_str);

        if validate_base58_address(&candidate_address) {
            let _ = tx.send(Message::Found {
                candidate: candidate_address,
            });
            return;
        }

        combinations_checked += 1;

        if combinations_checked % 800_000 == 0 {
            let elapsed = start_time.elapsed().as_secs_f32();
            let progress = (combinations_checked as f32 / total_combinations as f32) * 100.0;
            let remaining_time = if progress > 0.0 {
                (elapsed / progress) * (100.0 - progress)
            } else {
                0.0
            };
            let hashes_per_second = combinations_checked as f32 / elapsed;
            let progress_update = Message::ProgressUpdate {
                thread_id,
                progress,
                time_remaining: format!("{:.2} minutes", remaining_time / 60.0),
                hashes_per_second: format!("{:.2}", hashes_per_second),
                current_candidate: candidate_address.clone(),
            };

            let _ = tx.send(progress_update);
        }
    }

    let _ = tx.send(Message::Finished);
}

fn validate_base58_address(address: &str) -> bool {
    Address::from_str(address).is_ok()
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use base58::{FromBase58, ToBase58};
    use std::sync::{Arc, atomic::AtomicBool};

    /// Helper function to corrupt the checksum of a valid address.
    fn corrupt_checksum(address: &str) -> String {
        let decoded = address.from_base58().expect("Decoding failed");
        if decoded.len() != 25 {
            panic!("Address length is not 25 bytes");
        }

        // Split payload and checksum
        let (payload, checksum) = decoded.split_at(decoded.len() - 4);

        // Corrupt the checksum by flipping the first bit of the first checksum byte
        let mut corrupted_checksum = checksum.to_vec();
        corrupted_checksum[0] ^= 0x01;

        // Reconstruct the corrupted address
        let mut corrupted = Vec::new();
        corrupted.extend_from_slice(payload);
        corrupted.extend_from_slice(&corrupted_checksum);

        corrupted.to_base58()
    }

    #[test]
    fn test_validate_base58_address_valid() {
        let valid_address = "1BitcoinEaterAddressDontSendf59kuE";
        assert!(validate_base58_address(valid_address));
    }

    #[test]
    fn test_validate_base58_address_invalid_length() {
        let short_address = "1BitcoinEaterAddress"; // Too short
        assert!(!validate_base58_address(short_address));
    }

    #[test]
    fn test_validate_base58_address_invalid_characters() {
        let invalid_address = "1BitcoinEaterAddress!@#"; // Contains invalid characters
        assert!(!validate_base58_address(invalid_address));
    }

    #[test]
    fn test_validate_base58_address_invalid_checksum() {
        let valid_address = "1BitcoinEaterAddressDontSendf59kuE";
        let corrupted_address = corrupt_checksum(valid_address);
        assert!(!validate_base58_address(&corrupted_address));
    }

    #[test]
    fn test_start_index_calculation_valid_suffix() {
        let base58_alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        let start_suffix = "1A";
        let expected_index = base58_alphabet
            .find('A')
            .unwrap() as u128
            + base58_alphabet.find('1').unwrap() as u128 * base58_alphabet.len() as u128;

        let mut calculated_index = 0u128;
        for (idx, char) in start_suffix.chars().rev().enumerate() {
            if let Some(pos) = base58_alphabet.find(char) {
                let base58_len_pow = (base58_alphabet.len() as u128).pow(idx as u32);
                calculated_index += pos as u128 * base58_len_pow;
            }
        }

        assert_eq!(calculated_index, expected_index);
    }

    #[test]
    fn test_start_index_calculation_invalid_suffix() {
        let base58_alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        let invalid_suffix = "1A@"; // Invalid '@' character
        let mut index_result = None;

        for char in invalid_suffix.chars() {
            if base58_alphabet.find(char).is_none() {
                index_result = Some(format!("Invalid character in suffix: {}", char));
                break;
            }
        }

        assert_eq!(
            index_result,
            Some("Invalid character in suffix: @".to_string())
        );
    }

    #[test]
    fn test_progress_update_logic() {
        let total_combinations = 1_000_000u128;
        let combinations_checked = 500_000u128;
        let elapsed_time = 10.0; // seconds

        let expected_progress = (combinations_checked as f32 / total_combinations as f32) * 100.0;
        let expected_hashes_per_second = combinations_checked as f32 / elapsed_time;
        let remaining_time = (elapsed_time / expected_progress) * (100.0 - expected_progress);

        assert_eq!(expected_progress, 50.0);
        assert_eq!(expected_hashes_per_second, 50_000.0);
        assert!(remaining_time > 0.0);
    }

    #[test]
    fn test_brute_force_cancel_signal() {
        let stop_flag = Arc::new(AtomicBool::new(false));

        // Simulate cancelling the operation
        stop_flag.store(true, Ordering::SeqCst);
        assert!(stop_flag.load(Ordering::SeqCst));

        // Reset the flag
        stop_flag.store(false, Ordering::SeqCst);
        assert!(!stop_flag.load(Ordering::SeqCst));
    }

    #[test]
    fn test_message_flow_for_found_candidate() {
        let valid_address = "1BitcoinEaterAddressDontSendf59kuE".to_string();
        let (tx, rx): (Sender<Message>, Receiver<Message>) = bounded(1);

        // Send a "found" message
        let _ = tx.send(Message::Found {
            candidate: valid_address.clone(),
        });

        // Check received message
        if let Ok(Message::Found { candidate }) = rx.try_recv() {
            assert_eq!(candidate, valid_address);
        } else {
            panic!("Expected Message::Found but received something else.");
        }
    }

    #[test]
    fn test_message_flow_for_progress_update() {
        let (tx, rx): (Sender<Message>, Receiver<Message>) = bounded(1);

        // Send a progress update
        let progress_update = Message::ProgressUpdate {
            thread_id: 1,
            progress: 50.0,
            time_remaining: "10m".to_string(),
            hashes_per_second: "5000 H/s".to_string(),
            current_candidate: "1BitcoinEaterAddressDontSendf59kuE".to_string(),
        };
        let _ = tx.send(progress_update);

        // Check received message
        if let Ok(Message::ProgressUpdate {
            thread_id,
            progress,
            time_remaining,
            hashes_per_second,
            current_candidate,
        }) = rx.try_recv()
        {
            assert_eq!(thread_id, 1);
            assert_eq!(progress, 50.0);
            assert_eq!(time_remaining, "10m");
            assert_eq!(hashes_per_second, "5000 H/s");
            assert_eq!(
                current_candidate,
                "1BitcoinEaterAddressDontSendf59kuE".to_string()
            );
        } else {
            panic!("Expected Message::ProgressUpdate but received something else.");
        }
    }

    #[test]
    fn test_brute_force_logic_with_invalid_input() {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let base58_input = "InvalidInput".to_string(); // Too short
        let start_suffix = "1".to_string();
        let (tx, rx): (Sender<Message>, Receiver<Message>) = bounded(1);

        brute_force_checksum(base58_input, start_suffix, 0, tx, stop_flag);

        if let Ok(Message::Error(err)) = rx.try_recv() {
            assert_eq!(err, "Input address already 34 characters or longer.".to_string());
        } else {
            panic!("Expected Message::Error but received something else.");
        }
    }
}