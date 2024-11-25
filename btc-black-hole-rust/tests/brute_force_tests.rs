// tests/brute_force_tests.rs

use btc_black_hole_rust::{Message, ThreadInfo};
use crossbeam::channel::{bounded, Receiver, Sender};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use base58::{FromBase58, ToBase58};

#[test]
fn test_base58_encoding_decoding() {
    let address = "1BitcoinEaterAddressDontSendf59kuE";
    let decoded = address.from_base58().expect("Decoding failed");
    let encoded = decoded.to_base58();
    assert_eq!(address, encoded);
}

#[test]
fn test_invalid_base58_decoding() {
    let invalid_address = "0BitcoinEaterAddressDontSendf59kuE";
    assert!(invalid_address.from_base58().is_err());
}

#[test]
fn test_corrupted_base58_encoding() {
    let address = "1BitcoinEaterAddressDontSendf59kuE";
    let mut decoded = address.from_base58().expect("Decoding failed");
    decoded[0] ^= 0xFF; // Corrupt the data
    let corrupted = decoded.to_base58();
    assert_ne!(address, corrupted);
}

#[test]
fn test_progress_update_message() {
    let thread_id = 1;
    let progress = 50.0;
    let hashes_per_second = 5000.0;
    let current_candidate = "1BitcoinEaterAddressDontBendXYZ".to_string();
    let remaining_calculations = 500;
    let start_range_base58 = "StartBase58".to_string();
    let end_range_base58 = "EndBase58".to_string();

    // Create a ProgressUpdate message
    let message = Message::ProgressUpdate {
        thread_id,
        progress,
        hashes_per_second,
        current_candidate: current_candidate.clone(),
        remaining_calculations,
        start_range_base58: start_range_base58.clone(),
        end_range_base58: end_range_base58.clone(),
    };

    // Assert message fields
    if let Message::ProgressUpdate {
        thread_id: tid,
        progress: prog,
        hashes_per_second: hps,
        current_candidate: cc,
        remaining_calculations: rc,
        start_range_base58: srb58,
        end_range_base58: erb58,
    } = message
    {
        assert_eq!(tid, thread_id);
        assert_eq!(prog, progress);
        assert_eq!(hps, hashes_per_second);
        assert_eq!(cc, current_candidate);
        assert_eq!(rc, remaining_calculations);
        assert_eq!(srb58, start_range_base58);
        assert_eq!(erb58, end_range_base58);
    } else {
        panic!("Message is not of type ProgressUpdate");
    }
}

#[test]
fn test_message_flow_for_found_candidate() {
    let (tx, rx): (Sender<Message>, Receiver<Message>) = bounded(1);
    let candidate = "1BitcoinEaterAddressDontBendXYZ".to_string();

    // Send a Found message
    tx.send(Message::Found { candidate: candidate.clone() }).unwrap();

    // Receive and assert the message
    if let Ok(Message::Found { candidate: found_candidate }) = rx.try_recv() {
        assert_eq!(found_candidate, candidate);
    } else {
        panic!("Did not receive the expected Found message");
    }
}

#[test]
fn test_message_flow_for_finished() {
    let (tx, rx): (Sender<Message>, Receiver<Message>) = bounded(1);

    // Send a Finished message
    tx.send(Message::Finished).unwrap();

    // Receive and assert the message
    if let Ok(Message::Finished) = rx.try_recv() {
        assert!(true);
    } else {
        panic!("Did not receive the expected Finished message");
    }
}

#[test]
fn test_thread_info_initialization() {
    let thread_info = ThreadInfo {
        thread_id: 1,
        start_range_base58: "StartBase58".to_string(),
        end_range_base58: "EndBase58".to_string(),
        current_candidate: "1BitcoinEaterAddressDontBendXYZ".to_string(),
        remaining_calculations: 500,
    };

    assert_eq!(thread_info.thread_id, 1);
    assert_eq!(thread_info.start_range_base58, "StartBase58");
    assert_eq!(thread_info.end_range_base58, "EndBase58");
    assert_eq!(thread_info.current_candidate, "1BitcoinEaterAddressDontBendXYZ");
    assert_eq!(thread_info.remaining_calculations, 500);
}

#[test]
fn test_brute_force_cancellation() {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = Arc::clone(&stop_flag);

    stop_flag.store(true, Ordering::SeqCst);
    assert!(stop_flag_clone.load(Ordering::SeqCst));

    stop_flag.store(false, Ordering::SeqCst);
    assert!(!stop_flag_clone.load(Ordering::SeqCst));
}