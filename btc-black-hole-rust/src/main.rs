use bs58::decode;
use clap::Parser;
use crossbeam::channel::unbounded;
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Bitcoin Address Checksum Brute-Force
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Base58 Address without checksum
    #[arg(short, long)]
    address: Option<String>,

    /// Starting suffix (Base58)
    #[arg(short, long)]
    start_suffix: Option<String>,

    /// Number of threads to use
    #[arg(short, long)]
    threads: Option<usize>,
}

fn sha256d(data: &[u8]) -> [u8; 32] {
    let hash = Sha256::digest(data);
    Sha256::digest(&hash).into()
}

fn validate_base58_address(address: &str) -> bool {
    if let Ok(decoded) = decode(address).into_vec() {
        if decoded.len() != 25 {
            return false;
        }
        let (payload, checksum) = decoded.split_at(21);
        let hash = sha256d(payload);
        checksum == &hash[..4]
    } else {
        false
    }
}

fn generate_combinations(
    base58_input: String,
    chars_to_add: usize,
    start_index: u128,
    total_combinations: u128,
    progress: Arc<AtomicU64>,
    stop_flag: Arc<AtomicBool>,
    result_tx: crossbeam::channel::Sender<String>,
) {
    let base58_alphabet = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let base = base58_alphabet.len() as u128;

    let mut i = start_index;
    let end = total_combinations;

    while i < end && !stop_flag.load(Ordering::Relaxed) {
        // Convert i to Base58 string
        let mut temp = i;
        let mut suffix = vec![0u8; chars_to_add];
        for j in (0..chars_to_add).rev() {
            suffix[j] = base58_alphabet[(temp % base) as usize];
            temp /= base;
        }
        let candidate_address = format!("{}{}", base58_input, String::from_utf8(suffix).unwrap());

        if validate_base58_address(&candidate_address) {
            stop_flag.store(true, Ordering::Relaxed);
            let _ = result_tx.send(candidate_address);
            return;
        }

        i += 1;
        progress.fetch_add(1, Ordering::Relaxed);
    }
}

fn main() {
    let args = Args::parse();

    // Handle command-line or interactive inputs
    let address = args.address.unwrap_or_else(|| {
        print!("Enter Base58 Address (without checksum): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        input.trim().to_string()
    });

    let start_suffix = args.start_suffix.unwrap_or_else(|| {
        print!("Enter Starting Suffix (Base58) [Leave empty for default]: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        input.trim().to_string()
    });

    let threads = args.threads.unwrap_or_else(|| {
        print!("Enter Number of Threads [Default: CPU cores]: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        if let Ok(value) = input.trim().parse::<usize>() {
            value
        } else {
            num_cpus::get() // Default to CPU cores if parsing fails
        }
    });

    // Validate input address characters
    let base58_alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    if !address.chars().all(|c| base58_alphabet.contains(c)) {
        eprintln!("Error: Invalid character found in the input address.");
        return;
    }

    if !start_suffix.chars().all(|c| base58_alphabet.contains(c)) {
        eprintln!("Error: Invalid character found in the starting suffix.");
        return;
    }

    let current_length = address.len();
    if current_length >= 34 {
        eprintln!("Error: Input address is already 34 characters or longer.");
        return;
    }

    let chars_to_add = 34 - current_length;

    // Calculate starting index based on start_suffix
    let base = base58_alphabet.len() as u128;
    let mut start_index = 0u128;
    for (idx, c) in start_suffix.chars().rev().enumerate() {
        if let Some(pos) = base58_alphabet.chars().position(|x| x == c) {
            start_index += pos as u128 * base.pow(idx as u32);
        } else {
            eprintln!("Error: Invalid character '{}' in starting suffix.", c);
            return;
        }
    }

    let total_combinations = base.pow(chars_to_add as u32);
    if start_index >= total_combinations {
        eprintln!("Error: Starting suffix is beyond the total combination range.");
        return;
    }

    let progress = Arc::new(AtomicU64::new(0));
    let stop_flag = Arc::new(AtomicBool::new(false));

    let (result_tx, result_rx) = unbounded();

    // Split work among threads
    let combinations_per_thread = (total_combinations - start_index) / threads as u128;
    let mut handles = Vec::new();

    let start_time = Instant::now();

    for thread_id in 0..threads {
        let base58_input = address.clone();
        let progress = Arc::clone(&progress);
        let stop_flag = Arc::clone(&stop_flag);
        let result_tx = result_tx.clone();

        let thread_start = start_index + combinations_per_thread * thread_id as u128;
        let thread_end = if thread_id == threads - 1 {
            total_combinations
        } else {
            thread_start + combinations_per_thread
        };

        let handle = thread::spawn(move || {
            generate_combinations(
                base58_input,
                chars_to_add,
                thread_start,
                thread_end,
                progress,
                stop_flag,
                result_tx,
            );
        });

        handles.push(handle);
    }

    // Drop the extra sender
    drop(result_tx);

    // Monitor progress
    loop {
        thread::sleep(Duration::from_secs(1));

        let checked = progress.load(Ordering::Relaxed) as u128;
        let elapsed = start_time.elapsed().as_secs_f64();
        let hashes_per_sec = if elapsed > 0.0 {
            checked as f64 / elapsed
        } else {
            0.0
        };
        let progress_pct = (checked as f64 / (total_combinations - start_index) as f64) * 100.0;
        let remaining = if hashes_per_sec > 0.0 {
            ((total_combinations - start_index - checked) as f64 / hashes_per_sec) / 60.0
        } else {
            0.0
        };

        println!(
            "Progress: {:.6}% | Hashes/s: {:.2} | Time Remaining: {:.2} minutes",
            progress_pct, hashes_per_sec, remaining
        );

        if stop_flag.load(Ordering::Relaxed) {
            break;
        }

        if let Ok(result) = result_rx.try_recv() {
            println!("Valid address found: {}", result);
            break;
        }

        // Optionally, exit if all combinations are checked
        if checked >= (total_combinations - start_index) {
            break;
        }
    }

    // Wait for all threads to finish
    for handle in handles {
        let _ = handle.join();
    }

    // Final message if no address found
    if !stop_flag.load(Ordering::Relaxed) {
        println!("No valid address found.");
    }
}