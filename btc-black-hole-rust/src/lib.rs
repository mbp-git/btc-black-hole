// src/lib.rs

use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

use crossbeam::channel::{bounded, Receiver, Sender};
use eframe::egui;
use eframe::egui::{CentralPanel, ProgressBar, TextEdit, Vec2};
use num_cpus;
use bitcoin::Address;
use std::str::FromStr;

// Removed unused imports
// use base64::engine::general_purpose::STANDARD;
// use base64::Engine;
// use base58::{FromBase58, ToBase58};
use base58::FromBase58;

use sha2::{Digest, Sha256}; // Add this import at the top if not already present

// Define constants
const MAX_SUFFIX_LENGTH: usize = 34;
const MAX_ADDRESS_LENGTH: usize = 34; // Bitcoin addresses are 34 characters long
const MAX_DECODING_LENGTH: usize = 25; // Decoded Bitcoin address length (version + payload + checksum)

/// Messages sent from brute-force threads to the GUI
#[derive(Debug, Clone)]
pub enum Message {  
    ProgressUpdate {
        thread_id: usize,
        progress: f32,
        hashes_per_second: f64,
        current_candidate: String,
        remaining_calculations: u128,
        start_range_base58: String,
        end_range_base58: String,
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
    pub thread_id: usize,
    pub start_range_base58: String,
    pub end_range_base58: String,
    pub current_candidate: String,
    pub remaining_calculations: u128,
}

/// Structure to hold the application state
pub struct BruteForceApp {
    pub base58_input: String,
    pub thread_count: usize,
    pub start_range_base58: String, // Added field for input box
    pub progress: f32,
    pub total_hashes_per_second: f64,
    pub running: bool,
    pub stop_flag: Arc<AtomicBool>,
    pub receiver: Option<Receiver<Message>>,
    pub thread_infos: BTreeMap<usize, ThreadInfo>,
    pub found_addresses: Vec<String>,
}

impl Default for BruteForceApp {
    fn default() -> Self {
        Self {
            base58_input: String::from("1BitcoinEaterAddressDontSend"),
            thread_count: num_cpus::get(),
            start_range_base58: String::new(), // Initialize to empty string
            progress: 0.0,
            total_hashes_per_second: 0.0,
            running: false,
            stop_flag: Arc::new(AtomicBool::new(false)),
            receiver: None,
            thread_infos: BTreeMap::new(),
            found_addresses: Vec::new(),
        }
    }
}

impl eframe::App for BruteForceApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            // Set background color
            ui.style_mut().visuals.window_fill = egui::Color32::from_rgb(30, 30, 30);
            ui.set_min_size(Vec2::new(1200.0, 800.0));

            // Input Fields
            ui.vertical(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label("Enter Base58 Address (without checksum):");
                    ui.add(TextEdit::singleline(&mut self.base58_input).desired_width(600.0));
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label("Starting Range (Base58):");
                    ui.add(TextEdit::singleline(&mut self.start_range_base58).desired_width(200.0));
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
            });

            ui.separator();

            // Thread Information
            ui.label("Thread Information:");
            egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                egui::Grid::new("thread_info_grid")
                    .striped(true)
                    .min_col_width(100.0)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.heading("Thread ID");
                        ui.heading("Start Range");
                        ui.heading("End Range");
                        ui.heading("Candidate Address");
                        ui.heading("Remaining");
                        ui.end_row();

                        for (_thread_id, info) in &self.thread_infos {
                            ui.label(format!("{}", info.thread_id));
                            ui.label(&info.start_range_base58);
                            ui.label(&info.end_range_base58);
                            ui.label(&info.current_candidate);
                            ui.label(format!("{}", info.remaining_calculations));
                            ui.end_row();
                        }
                    });
            });

            // Combined Progress Bar
            ui.add(ProgressBar::new(self.progress / 100.0).show_percentage());

            // Total Hashes per Second
            ui.label(format!(
                "Total Hashes per Second: {}",
                format_hash_rate(self.total_hashes_per_second)
            ));

            ui.separator();

            // Found Addresses
            ui.label("Found Addresses:");
            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                for addr in &self.found_addresses {
                    ui.label(addr);
                }
            });

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
                            hashes_per_second,
                            current_candidate,
                            remaining_calculations,
                            start_range_base58,
                            end_range_base58,
                        } => {
                            self.update_thread_info(
                                thread_id,
                                current_candidate,
                                remaining_calculations,
                                start_range_base58,
                                end_range_base58,
                            );
                            self.total_hashes_per_second = hashes_per_second;
                            self.progress = progress;
                        }
                        Message::Found { candidate } => {
                            self.found_addresses.push(candidate);
                        }
                        Message::Finished => {
                            self.running = false;
                            self.receiver = None;
                        }
                        Message::Cancelled => {
                            self.progress = 0.0;
                            self.running = false;
                            self.receiver = None;
                            self.thread_infos.clear();
                        }
                        Message::Error(err) => {
                            self.progress = 0.0;
                            self.running = false;
                            self.receiver = None;
                            self.thread_infos.clear();
                            println!("Error: {}", err);
                        }
                    }
                }
            }

            if self.running {
                ctx.request_repaint_after(Duration::from_secs(1));
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
        self.progress = 0.0;
        self.total_hashes_per_second = 0.0;
        self.thread_infos.clear();
        self.found_addresses.clear();

        let (tx, rx): (Sender<Message>, Receiver<Message>) = bounded(100);
        self.receiver = Some(rx);

        // Determine the number of characters to add
        let current_length = base58_input.len();
        if current_length >= 34 {
            println!("Input address already 34 characters or longer.");
            self.running = false;
            return;
        }
        let chars_to_add = 34 - current_length;

        if chars_to_add > 10 {
            println!("Input size too large; potential overflow.");
            self.running = false;
            return;
        }

        let base58_alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        let base58_len = base58_alphabet.len() as u128;
        let total_combinations = (base58_len)
            .checked_pow(chars_to_add as u32)
            .unwrap_or(u128::MAX);

        // Parse start_range_base58 into start_index
        let start_index = if !self.start_range_base58.is_empty() {
            match self.start_range_base58.from_base58() {
                Ok(bytes) => {
                    let mut arr = [0u8; 16]; // u128 is 16 bytes
                    let bytes_len = bytes.len().min(16);
                    arr[16 - bytes_len..].copy_from_slice(&bytes[..bytes_len]);
                    u128::from_be_bytes(arr)
                }
                Err(_) => {
                    println!("Invalid start range base58 input.");
                    self.running = false;
                    return;
                }
            }
        } else {
            0u128
        };

        let combinations_per_thread = (total_combinations - start_index) / thread_count as u128;
        let mut thread_start_index = start_index;

        for thread_id in 0..thread_count {
            let thread_end_index = if thread_id == thread_count - 1 {
                total_combinations
            } else {
                thread_start_index + combinations_per_thread
            };

            let base58_input_clone = base58_input.clone();
            let tx_clone = tx.clone();
            let stop_flag_clone = Arc::clone(&self.stop_flag);

            // Convert start and end ranges to base58
            let start_range_base58 = base58_encode(thread_start_index, chars_to_add);
            let end_range_base58 = base58_encode(thread_end_index - 1, chars_to_add);

            self.thread_infos.insert(
                thread_id,
                ThreadInfo {
                    thread_id,
                    start_range_base58: start_range_base58.clone(),
                    end_range_base58: end_range_base58.clone(),
                    current_candidate: String::new(),
                    remaining_calculations: thread_end_index - thread_start_index,
                },
            );

            thread::spawn(move || {
                brute_force_checksum(
                    &base58_input_clone,
                    thread_start_index,
                    thread_end_index,
                    thread_id,
                    tx_clone,
                    stop_flag_clone,
                )
            });

            thread_start_index = thread_end_index;
        }
    }

    fn update_thread_info(
        &mut self,
        thread_id: usize,
        current_candidate: String,
        remaining_calculations: u128,
        start_range_base58: String,
        end_range_base58: String,
    ) {
        if let Some(info) = self.thread_infos.get_mut(&thread_id) {
            info.current_candidate = current_candidate;
            info.remaining_calculations = remaining_calculations;
            info.start_range_base58 = start_range_base58;
            info.end_range_base58 = end_range_base58;
        }
    }
}

pub fn brute_force_checksum(
    base58_input: &str,
    start_range: u128,
    end_range: u128,
    thread_id: usize,
    tx: Sender<Message>,
    stop_flag: Arc<AtomicBool>,
) {
    let chars_to_add = 34 - base58_input.len();

    let total = end_range - start_range;
    let mut last_update = Instant::now();
    let mut hashes_done = 0u128;
    let mut i = start_range;

    // Pre-allocate buffers
    let mut suffix_chars = [0u8; MAX_SUFFIX_LENGTH];
    let mut candidate_address = [0u8; MAX_ADDRESS_LENGTH]; // Bitcoin addresses are 34 characters long
    let base58_input_bytes = base58_input.as_bytes();
    candidate_address[..base58_input_bytes.len()].copy_from_slice(base58_input_bytes);

    let base58_alphabet_bytes = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let base = 58u128;

    while i < end_range {
        if stop_flag.load(Ordering::SeqCst) {
            let _ = tx.send(Message::Cancelled);
            return;
        }

        // Base58 encode counter value into suffix_chars
        let mut value = i;
        for idx in (0..chars_to_add).rev() {
            let rem = (value % base) as usize;
            suffix_chars[idx] = base58_alphabet_bytes[rem];
            value /= base;
        }

        // Construct candidate address without allocations
        candidate_address[base58_input_bytes.len()..].copy_from_slice(&suffix_chars[..chars_to_add]);

        // Validate address
        if validate_base58_address(&candidate_address[..base58_input_bytes.len() + chars_to_add]) {
            let found_address = String::from_utf8_lossy(
                &candidate_address[..base58_input_bytes.len() + chars_to_add],
            )
            .to_string();
            let _ = tx.send(Message::Found { candidate: found_address });
        }

        hashes_done += 1;

        if last_update.elapsed() >= Duration::from_secs(1) {
            let progress = ((i - start_range) as f32 / total as f32) * 100.0;
            let hashes_per_second = hashes_done as f64 / last_update.elapsed().as_secs_f64();
            let remaining_calculations = end_range - i;

            let _ = tx.send(Message::ProgressUpdate {
                thread_id,
                progress,
                hashes_per_second,
                current_candidate: String::from_utf8_lossy(
                    &candidate_address[..base58_input_bytes.len() + chars_to_add],
                )
                .to_string(),
                remaining_calculations,
                start_range_base58: base58_encode(start_range, chars_to_add),
                end_range_base58: base58_encode(end_range - 1, chars_to_add),
            });

            last_update = Instant::now();
            hashes_done = 0;
        }

        i += 1;
    }

    let _ = tx.send(Message::Finished);
}

pub fn validate_base58_address(address: &[u8]) -> bool {
    // Decode Base58 to bytes
    let mut decoded = [0u8; MAX_DECODING_LENGTH];
    match base58_decode(address, &mut decoded) {
        Ok(decoded_len) => {
            if decoded_len != 25 {
                return false;
            }

            // Split payload and checksum
            let (payload, checksum) = decoded.split_at(decoded_len - 4);

            // Compute checksum
            let computed_checksum = double_sha256(payload);

            // Compare checksums
            checksum == &computed_checksum[..4]
        }
        Err(_) => false,
    }
}

// Implement a simple Base58 decoder that writes into a fixed-size buffer
fn base58_decode(input: &[u8], output: &mut [u8]) -> Result<usize, ()> {
    let mut result = [0u8; 32]; // Temporary buffer for result
    let mut result_len = 0;

    for &byte in input {
        let mut carry = match BASE58_DECODE_MAP[byte as usize] {
            255 => return Err(()), // Invalid character
            val => val as u32,
        };

        for i in 0..result_len {
            let val = result[i] as u32 * 58 + carry;
            result[i] = (val & 0xFF) as u8;
            carry = val >> 8;
        }

        while carry > 0 {
            result[result_len] = (carry & 0xFF) as u8;
            result_len += 1;
            carry >>= 8;
        }
    }

    // Leading zeros
    let mut leading_zeros = 0;
    for &byte in input {
        if byte == b'1' {
            leading_zeros += 1;
        } else {
            break;
        }
    }

    // Copy result to output buffer in big-endian order
    let total_len = leading_zeros + result_len;
    if total_len > output.len() {
        return Err(());
    }

    output[..leading_zeros].fill(0);
    for i in 0..result_len {
        output[total_len - 1 - i] = result[i];
    }

    Ok(total_len)
}

// Double SHA256 hash function
fn double_sha256(data: &[u8]) -> [u8; 32] {
    let first_hash = Sha256::digest(data);
    let second_hash = Sha256::digest(&first_hash);
    let mut result = [0u8; 32];
    result.copy_from_slice(&second_hash);
    result
}

// Precompute Base58 decode map
const BASE58_DECODE_MAP: [u8; 256] = {
    let mut map = [255u8; 256];
    let alphabet = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut i = 0;
    while i < alphabet.len() {
        map[alphabet[i] as usize] = i as u8;
        i += 1;
    }
    map
};

fn base58_encode(mut value: u128, length: usize) -> String {
    let base58_alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut result = vec![0u8; length];
    let base = base58_alphabet.len() as u128;

    for idx in (0..length).rev() {
        let rem = (value % base) as usize;
        result[idx] = base58_alphabet.as_bytes()[rem];
        value /= base;
    }

    unsafe { String::from_utf8_unchecked(result) }
}

// Utility function to format hash rate
fn format_hash_rate(hashes_per_second: f64) -> String {
    if hashes_per_second >= 1_000_000_000_000.0 {
        format!("{:.2} TH/s", hashes_per_second / 1_000_000_000_000.0)
    } else if hashes_per_second >= 1_000_000_000.0 {
        format!("{:.2} GH/s", hashes_per_second / 1_000_000_000.0)
    } else if hashes_per_second >= 1_000_000.0 {
        format!("{:.2} MH/s", hashes_per_second / 1_000_000.0)
    } else if hashes_per_second >= 1_000.0 {
        format!("{:.2} KH/s", hashes_per_second / 1_000.0)
    } else {
        format!("{:.2} H/s", hashes_per_second)
    }
}