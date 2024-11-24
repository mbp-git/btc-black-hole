// tests/brute_force_tests.rs

use btc_black_hole_rust::*;
use base58::{FromBase58, ToBase58};
use std::sync::{Arc, atomic::AtomicBool, atomic::Ordering};
use crossbeam::channel::{bounded, Sender, Receiver};

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
        start_range: 0,
        end_range: 1000,
    };
    let _ = tx.send(progress_update);

    // Check received message
    if let Ok(Message::ProgressUpdate {
        thread_id,
        progress,
        time_remaining,
        hashes_per_second,
        current_candidate,
        start_range,
        end_range,
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
        assert_eq!(start_range, 0);
        assert_eq!(end_range, 1000);
    } else {
        panic!("Expected Message::ProgressUpdate but received something else.");
    }
}

// #[test]
// fn test_brute_force_logic_with_invalid_input() {
//     let stop_flag = Arc::new(AtomicBool::new(false));
//     let base58_input = "1".to_string(); // Too short to complete a valid address
//     let (tx, rx): (Sender<Message>, Receiver<Message>) = bounded(1);

//     // Since start_bruteforce assigns ranges, directly calling brute_force_checksum
//     brute_force_checksum(base58_input, 0, 1000, 0, tx, stop_flag);

//     if let Ok(Message::Error(err)) = rx.try_recv() {
//         assert_eq!(
//             err,
//             "Invalid input length for a Bitcoin address.".to_string()
//         );
//     } else {
//         panic!("Expected Message::Error but received something else.");
//     }
// }