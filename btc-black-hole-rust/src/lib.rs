use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Instant;

use crossbeam::channel::{bounded, Receiver, Sender};
use eframe::egui;
use eframe::egui::{CentralPanel, ProgressBar, TextEdit, Vec2};
use num_cpus;
use bitcoin::Address;
use std::str::FromStr;

/// Messages sent from brute-force threads to the GUI
#[derive(Debug, Clone)]
pub enum Message {
    ProgressUpdate {
        thread_id: usize,
        progress: f32,
        time_remaining: String,
        hashes_per_second: String,
        current_candidate: String,
        start_range: u128,
        end_range: u128,
    },
    Found {
        candidate: String,
    },
    Finished,
    Cancelled,
    Error(String),
}

/// Structure to hold information about each thread
#[derive(Debug, Clone)]
pub struct ThreadInfo {
    pub last_candidate: String,
    pub start_range: u128,
    pub end_range: u128,
}

/// Structure to hold the application state
pub struct BruteForceApp {
    pub base58_input: String,
    pub start_suffix: String,
    pub thread_count: usize,
    pub progress: f32,
    pub time_remaining: String,
    pub hashes_per_second: String,
    pub result: String,
    pub running: bool,
    pub stop_flag: Arc<AtomicBool>,
    pub receiver: Option<Receiver<Message>>,
    pub thread_infos: HashMap<usize, ThreadInfo>,
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
            thread_infos: HashMap::new(),
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

            // Thread Information
            ui.label("Thread Information:");
            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                egui::Grid::new("thread_info_grid")
                    .striped(true)
                    .min_col_width(100.0)
                    .show(ui, |ui| {
                        ui.heading("Thread ID");
                        ui.heading("Last Candidate");
                        ui.heading("Start Range");
                        ui.heading("End Range");
                        ui.end_row();

                        for (thread_id, info) in &self.thread_infos {
                            ui.label(format!("{}", thread_id));
                            ui.label(&info.last_candidate);
                            ui.label(format!("{}", info.start_range));
                            ui.label(format!("{}", info.end_range));
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
                            start_range,
                            end_range,
                        } => {
                            self.progress = progress;
                            self.time_remaining = time_remaining;
                            self.hashes_per_second = hashes_per_second;
                            self.thread_infos.insert(
                                thread_id,
                                ThreadInfo {
                                    last_candidate: current_candidate,
                                    start_range,
                                    end_range,
                                },
                            );
                        }
                        Message::Found { candidate } => {
                            self.result = format!("Valid address found: {}", candidate);
                            self.progress = 100.0;
                            self.time_remaining = "Completed".to_string();
                            self.hashes_per_second = "N/A".to_string();
                            self.running = false;
                            self.receiver = None;
                            self.thread_infos.clear();
                        }
                        Message::Finished => {
                            self.result = "No valid address found.".to_string();
                            self.progress = 100.0;
                            self.time_remaining = "N/A".to_string();
                            self.hashes_per_second = "N/A".to_string();
                            self.running = false;
                            self.receiver = None;
                            self.thread_infos.clear();
                        }
                        Message::Cancelled => {
                            self.result = "Brute-forcing cancelled.".to_string();
                            self.progress = 0.0;
                            self.time_remaining = "N/A".to_string();
                            self.hashes_per_second = "N/A".to_string();
                            self.running = false;
                            self.receiver = None;
                            self.thread_infos.clear();
                        }
                        Message::Error(err) => {
                            self.result = format!("Error: {}", err);
                            self.running = false;
                            self.receiver = None;
                            self.thread_infos.clear();
                        }
                    }
                }
            }

            if self.running {
                ctx.request_repaint();
            }
        });
    }
}

impl BruteForceApp {
    pub fn start_bruteforce(&mut self) {
        self.stop_flag.store(false, Ordering::SeqCst);

        let base58_input = self.base58_input.clone();
        let thread_count = self.thread_count;
        self.running = true;
        self.result.clear();
        self.progress = 0.0;
        self.time_remaining = "Calculating...".to_string();
        self.hashes_per_second = "Calculating...".to_string();
        self.thread_infos.clear();

        let (tx, rx): (Sender<Message>, Receiver<Message>) = bounded(100);
        self.receiver = Some(rx);

        // Determine the number of characters to add
        let current_length = base58_input.len();
        if current_length >= 34 {
            self.result = "Input address already 34 characters or longer.".to_string();
            self.running = false;
            return;
        }
        let chars_to_add = 34 - current_length;

        if chars_to_add > 10 {
            self.result = "Input size too large; potential overflow.".to_string();
            self.running = false;
            return;
        }

        let base58_alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        let base58_len = base58_alphabet.len() as u128;
        let total_combinations = base58_len.checked_pow(chars_to_add as u32).unwrap_or(u128::MAX);

        let combinations_per_thread = total_combinations / thread_count as u128;
        let mut start_index = 0u128;

        for thread_id in 0..thread_count {
            let end_index = if thread_id == thread_count - 1 {
                total_combinations
            } else {
                start_index + combinations_per_thread
            };

            let base58_input_clone = base58_input.clone();
            let tx_clone = tx.clone();
            let stop_flag_clone = Arc::clone(&self.stop_flag);

            thread::spawn(move || {
                brute_force_checksum(
                    base58_input_clone,
                    start_index,
                    end_index,
                    thread_id,
                    tx_clone,
                    stop_flag_clone,
                )
            });

            self.thread_infos.insert(
                thread_id,
                ThreadInfo {
                    last_candidate: String::new(),
                    start_range: start_index,
                    end_range: end_index,
                },
            );

            start_index = end_index;
        }
    }
}

pub fn brute_force_checksum(
    base58_input: String,
    start_range: u128,
    end_range: u128,
    thread_id: usize,
    tx: Sender<Message>,
    stop_flag: Arc<AtomicBool>,
) {
    let start_time = Instant::now();
    let base58_alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let base58_len = base58_alphabet.len() as u128;
    let chars_to_add = 34 - base58_input.len();

    if base58_input.len() + chars_to_add != 34 {
        let _ = tx.send(Message::Error(
            "Invalid input length for a Bitcoin address.".to_string(),
        ));
        return;
    }

    for i in start_range..end_range {
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

        if i % 800_000 == 0 {
            let elapsed = start_time.elapsed().as_secs_f32();
            let progress = (i as f32 / end_range as f32) * 100.0;
            let remaining_time = if progress > 0.0 {
                (elapsed / progress) * (100.0 - progress)
            } else {
                0.0
            };
            let hashes_per_second = i as f32 / elapsed;
            let progress_update = Message::ProgressUpdate {
                thread_id,
                progress,
                time_remaining: format!("{:.2} minutes", remaining_time / 60.0),
                hashes_per_second: format!("{:.2}", hashes_per_second),
                current_candidate: candidate_address.clone(),
                start_range,
                end_range,
            };

            let _ = tx.send(progress_update);
        }
    }

    let _ = tx.send(Message::Finished);
}

pub fn validate_base58_address(address: &str) -> bool {
    Address::from_str(address).is_ok()
}