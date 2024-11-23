use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Instant;

use base58::FromBase58;
use eframe::egui;
use eframe::egui::CentralPanel;
use sha2::{Digest, Sha256};
use crossbeam::channel::{bounded, Receiver, Sender};
use num_cpus;

/// Messages sent from brute-force threads to the GUI
enum Message {
    ProgressUpdate {
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

/// Perform double SHA-256 hashing.
fn sha256d(data: &[u8]) -> Vec<u8> {
    let first_hash = Sha256::digest(data);
    let second_hash = Sha256::digest(&first_hash);
    second_hash.to_vec()
}

/// Validate a Base58 Bitcoin address by checking the checksum.
fn validate_base58_address(address: &str) -> bool {
    match address.from_base58() {
        Ok(decoded) => {
            if decoded.len() < 4 {
                return false;
            }
            let (payload, checksum) = decoded.split_at(decoded.len() - 4);
            let calculated_checksum = &sha256d(payload)[..4];
            calculated_checksum == checksum
        },
        Err(_) => false,
    }
}

/// Structure to hold the application state.
struct BruteForceApp {
    base58_input: String,
    start_suffix: String,
    progress: f32,
    time_remaining: String,
    hashes_per_second: String,
    current_candidate: String,
    result: String,
    running: bool,
    stop_flag: Arc<AtomicBool>,
    receiver: Option<Receiver<Message>>,
}

impl Default for BruteForceApp {
    fn default() -> Self {
        Self {
            base58_input: String::from("1BitcoinEaterAddressDontSendf59kuE"),
            start_suffix: String::from("1"),
            progress: 0.0,
            time_remaining: String::from("Calculating..."),
            hashes_per_second: String::from("Calculating..."),
            current_candidate: String::new(),
            result: String::new(),
            running: false,
            stop_flag: Arc::new(AtomicBool::new(false)),
            receiver: None,
        }
    }
}

impl eframe::App for BruteForceApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            // Set background color
            ui.style_mut().visuals.window_fill = egui::Color32::from_rgb(30, 30, 30);
            ui.set_min_size(egui::Vec2::new(800.0, 720.0));

            // Input Fields
            ui.horizontal_wrapped(|ui| {
                ui.label("Enter Base58 Address (without checksum):");
                ui.text_edit_singleline(&mut self.base58_input);
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("Starting Suffix (Base58):");
                ui.text_edit_singleline(&mut self.start_suffix);
            });

            ui.separator();

            // Current Candidate Address
            ui.label(format!("Current Candidate Address: {}", self.current_candidate));
            ui.add(egui::ProgressBar::new(self.progress / 100.0).show_percentage());

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
                egui::TextEdit::multiline(&mut self.result)
                    .desired_rows(10)
                    .desired_width(f32::INFINITY),
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
                // Collect all available messages first
                let mut messages = Vec::new();
                while let Ok(message) = receiver.try_recv() {
                    messages.push(message);
                }

                for message in messages {
                    match message {
                        Message::ProgressUpdate { progress, time_remaining, hashes_per_second, current_candidate } => {
                            self.progress = progress;
                            self.time_remaining = time_remaining;
                            self.hashes_per_second = hashes_per_second;
                            self.current_candidate = current_candidate;
                        },
                        Message::Found { candidate } => {
                            self.result = format!("Valid address found: {}", candidate);
                            self.progress = 100.0;
                            self.time_remaining = "Completed".to_string();
                            self.hashes_per_second = "N/A".to_string();
                            self.current_candidate = candidate;
                            self.running = false;
                            self.receiver = None;
                        },
                        Message::Finished => {
                            self.result = "No valid address found.".to_string();
                            self.progress = 100.0;
                            self.time_remaining = "N/A".to_string();
                            self.hashes_per_second = "N/A".to_string();
                            self.running = false;
                            self.receiver = None;
                        },
                        Message::Cancelled => {
                            self.result = "Brute-forcing cancelled.".to_string();
                            self.progress = 0.0;
                            self.time_remaining = "N/A".to_string();
                            self.hashes_per_second = "N/A".to_string();
                            self.running = false;
                            self.receiver = None;
                        },
                        Message::Error(err) => {
                            self.result = format!("Error: {}", err);
                            self.running = false;
                            self.receiver = None;
                        },
                    }
                }
            }

            // Request a repaint if running to update UI
            if self.running {
                ctx.request_repaint();
            }
        });
    }
}

impl BruteForceApp {
    fn start_bruteforce(&mut self) {
        let base58_input = self.base58_input.clone();
        let start_suffix = self.start_suffix.clone();
        let stop_flag = Arc::clone(&self.stop_flag);
        self.running = true;
        self.result.clear();
        self.progress = 0.0;
        self.time_remaining = "Calculating...".to_string();
        self.hashes_per_second = "Calculating...".to_string();
        self.current_candidate.clear();

        // Create a channel for communication
        let (tx, rx): (Sender<Message>, Receiver<Message>) = bounded(100);

        // Store the receiver in the struct
        self.receiver = Some(rx);

        // Start the brute-force thread
        thread::spawn(move || {
            brute_force_checksum(
                base58_input,
                start_suffix,
                tx,
                stop_flag,
            )
        });
    }
}

/// Brute-force function optimized for performance.
fn brute_force_checksum(
    base58_input: String,
    start_suffix: String,
    tx: Sender<Message>,
    stop_flag: Arc<AtomicBool>,
) {
    let start_time = Instant::now();
    let base58_alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let base58_len = base58_alphabet.len() as u128;

    // Determine how many characters need to be added to make it 34 bytes
    let current_length = base58_input.len();
    if current_length >= 34 {
        tx.send(Message::Error("Input address already 34 characters or longer.".to_string())).unwrap();
        return;
    }

    let chars_to_add = 34 - current_length;

    // Generate the starting index based on the starting suffix
    let mut start_index = 0u128;
    for (idx, char) in start_suffix.chars().rev().enumerate() {
        if let Some(pos) = base58_alphabet.find(char) {
            // Correct type casting: base58_len is u128, pow returns u128
            let base58_len_pow = base58_len.pow(idx as u32);
            start_index += pos as u128 * base58_len_pow;
        } else {
            // Invalid character in starting suffix
            tx.send(Message::Error("Invalid character in starting suffix.".to_string())).unwrap();
            return;
        }
    }

    // Iterate over all combinations of Base58 characters for the required length
    let total_combinations = base58_len.pow(chars_to_add as u32);
    let mut combinations_checked = 0u128;

    // Utilize multiple threads for parallel processing
    let num_threads = num_cpus::get();
    let chunk_size = if num_threads > 0 {
        (total_combinations - start_index) / num_threads as u128
    } else {
        total_combinations
    };
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let base58_input = base58_input.clone();
        let base58_alphabet = base58_alphabet.to_string();
        let tx = tx.clone();
        let stop_flag = Arc::clone(&stop_flag);

        let start = start_index + thread_id as u128 * chunk_size;
        let end = if thread_id == num_threads - 1 {
            total_combinations
        } else {
            start + chunk_size
        };

        let handle = thread::spawn(move || {
            for i in start..end {
                if stop_flag.load(Ordering::SeqCst) {
                    tx.send(Message::Cancelled).unwrap();
                    return;
                }

                // Generate the current combination
                let mut temp = i;
                let mut suffix = Vec::with_capacity(chars_to_add);
                for _ in 0..chars_to_add {
                    let idx = (temp % base58_len) as usize;
                    suffix.push(base58_alphabet.chars().nth(idx).unwrap());
                    temp /= base58_len;
                }
                suffix.reverse();
                let suffix_str: String = suffix.into_iter().collect();

                // Create the candidate address
                let candidate_address = format!("{}{}", base58_input, suffix_str);

                // Validate the candidate address
                if validate_base58_address(&candidate_address) {
                    tx.send(Message::Found { candidate: candidate_address }).unwrap();
                    return;
                }

                combinations_checked += 1;

                // Update progress, ETC, and H/s in UI every 800,000 combinations
                if combinations_checked % 800_000 == 0 {
                    let elapsed = start_time.elapsed().as_secs_f32();
                    let progress = combinations_checked as f32 / total_combinations as f32 * 100.0;
                    let remaining_time = if progress > 0.0 {
                        (elapsed / progress) * (100.0 - progress)
                    } else {
                        0.0
                    };
                    let hashes_per_second = combinations_checked as f32 / elapsed;
                    let progress_update = Message::ProgressUpdate {
                        progress,
                        time_remaining: format!("{:.2} minutes", remaining_time / 60.0),
                        hashes_per_second: format!("{:.2}", hashes_per_second),
                        current_candidate: candidate_address.clone(),
                    };

                    tx.send(progress_update).unwrap();
                }
            }

            tx.send(Message::Finished).unwrap();
        });

        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // If no address found and not cancelled
    if !stop_flag.load(Ordering::SeqCst) {
        tx.send(Message::Finished).unwrap();
    }
}

fn main() {
    let app = BruteForceApp::default();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(900.0, 750.0)),
        ..Default::default()
    };
    eframe::run_native(
        "BTC Address Checksum Brute-Force",
        native_options,
        Box::new(|_cc| Box::new(app)),
    ).unwrap(); // Handle the Result by unwrapping
}