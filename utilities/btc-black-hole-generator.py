# https://bitcointalk.org/index.php?topic=136362.0
# Test address: 1BitcoinEaterAddressDontSendf59kuE
# Hex: 00759d6677091e973b9e9d99f19c68fbf43e3f05f95eabd8a1
# Payload: 759d6677091e973b9e9d99f19c68fbf43e3f05f9
# Checksum: eabd8a1

import tkinter as tk
from tkinter import ttk
from hashlib import sha256
import base58
import binascii
from threading import Thread, Event
import time
from itertools import product


def sha256d(data):
    """
    Perform double SHA-256 hashing.
    """
    return sha256(sha256(data).digest()).digest()


def validate_base58_address(address):
    """
    Validate a Base58 Bitcoin address by checking the checksum.
    """
    try:
        decoded = base58.b58decode(address)
        payload, checksum = decoded[:-4], decoded[-4:]
        if sha256d(payload)[:4] == checksum:
            return True
        return False
    except (ValueError, binascii.Error):
        return False


def brute_force_checksum(base58_input, start_suffix, progress_label, progress_bar, time_label, hps_label, payload_hex_textbox, stop_event):
    """
    Brute-force valid Base58 characters to make the total address 34 bytes and validate it.
    """
    start_time = time.time()
    base58_alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"

    # Determine how many characters need to be added to make it 34 bytes
    current_length = len(base58_input)
    if current_length >= 34:
        result_text.insert(tk.END, "Input address already 34 characters or longer.\n")
        return

    chars_to_add = 34 - current_length

    # Generate the starting point based on the starting suffix
    start_index = 0
    for idx, char in enumerate(reversed(start_suffix)):
        start_index += base58_alphabet.index(char) * (len(base58_alphabet) ** idx)

    # Iterate over all combinations of Base58 characters for the required length
    total_combinations = len(base58_alphabet) ** chars_to_add
    combinations_checked = 0

    for i in range(start_index, total_combinations):
        if stop_event.is_set():  # Check if cancel is triggered
            result_text.insert(tk.END, "Brute-forcing cancelled.\n")
            return

        # Generate the current combination
        combination = []
        temp = i
        for _ in range(chars_to_add):
            combination.append(base58_alphabet[temp % len(base58_alphabet)])
            temp //= len(base58_alphabet)
        suffix = ''.join(reversed(combination))

        # Create the candidate address
        candidate_address = base58_input + suffix

        # Validate the candidate address
        if validate_base58_address(candidate_address):
            result_text.insert(tk.END, f"Valid address found: {candidate_address}\n")
            progress_label.config(text="Progress: Done!")
            progress_bar["value"] = 100
            time_label.config(text="Time Remaining: Completed")
            hps_label.config(text="Hashes per Second: N/A")
            payload_hex_textbox.config(state="normal")
            payload_hex_textbox.delete("1.0", tk.END)
            payload_hex_textbox.insert("1.0", candidate_address)
            payload_hex_textbox.config(state="disabled")
            return

        # Update progress, ETC, and H/s in UI every 800,000 combinations
        combinations_checked += 1
        if combinations_checked % 800000 == 0:
            elapsed_time = time.time() - start_time
            progress = combinations_checked / total_combinations
            remaining_time = (elapsed_time / progress) - elapsed_time if progress > 0 else 0
            hashes_per_second = combinations_checked / elapsed_time if elapsed_time > 0 else 0

            # Update current candidate address in the GUI
            payload_hex_textbox.config(state="normal")
            payload_hex_textbox.delete("1.0", tk.END)
            payload_hex_textbox.insert("1.0", candidate_address)
            payload_hex_textbox.config(state="disabled")

            # Display progress
            progress_label.config(
                text=f"Progress: {progress * 100:.2f}% | Last Checked: {candidate_address}"
            )
            progress_bar["value"] = progress * 100
            time_label.config(
                text=f"Time Remaining: {remaining_time / 60:.2f} minutes" if progress > 0 else "Time Remaining: Calculating..."
            )
            hps_label.config(text=f"Hashes per Second: {hashes_per_second:.2f}")
            progress_bar.update()

    result_text.insert(tk.END, "No valid address found.\n")
    progress_label.config(text="Progress: Finished.")
    time_label.config(text="Time Remaining: N/A")
    hps_label.config(text="Hashes per Second: N/A")


def start_bruteforce():
    """
    Start the brute-forcing process and update the GUI.
    """
    global stop_event
    stop_event.clear()  # Reset the stop event

    base58_input = input_textbox.get("1.0", tk.END).strip()
    start_suffix = start_suffix_entry.get().strip()

    progress_label.config(text="Starting brute force...")
    progress_bar["value"] = 0

    # Run brute-force in a separate thread
    thread = Thread(
        target=brute_force_checksum,
        args=(base58_input, start_suffix, progress_label, progress_bar, time_label, hps_label, payload_hex_textbox, stop_event),
    )
    thread.start()


def cancel_bruteforce():
    """
    Cancel the brute-forcing process.
    """
    stop_event.set()  # Signal the stop event


# Create the stop_event for thread-safe cancellation
stop_event = Event()

# Create the main window
root = tk.Tk()
root.title("BTC Address Checksum Brute-Force")
root.geometry("800x720")
root.configure(bg="black")

# Input label and textbox
input_label = tk.Label(
    root, text="Enter Base58 Address (without checksum):", font=("Arial", 12), bg="black", fg="white"
)
input_label.pack(pady=10)

input_textbox = tk.Text(
    root,
    height=2,
    width=60,
    font=("Arial", 12),
    bg="gray10",
    fg="white",
    insertbackground="white",
)
input_textbox.pack(pady=10)

# Starting checksum
start_suffix_label = tk.Label(
    root, text="Starting Suffix (Base58):", font=("Arial", 12), bg="black", fg="white"
)
start_suffix_label.pack(pady=10)

start_suffix_entry = tk.Entry(
    root, font=("Arial", 12), bg="gray10", fg="white"
)
start_suffix_entry.insert(0, "1")  # Default starting suffix
start_suffix_entry.pack(pady=10)

# Current payload hex
payload_hex_label = tk.Label(
    root, text="Current Candidate Address (Read-Only):", font=("Arial", 12), bg="black", fg="white"
)
payload_hex_label.pack(pady=10)

payload_hex_textbox = tk.Text(
    root,
    height=2,
    width=60,
    font=("Arial", 12),
    bg="gray10",
    fg="white",
    state="disabled",  # Start in read-only mode
)
payload_hex_textbox.pack(pady=10)

# Progress bar and labels
progress_label = tk.Label(
    root, text="Progress: 0%", font=("Arial", 12), bg="black", fg="white"
)
progress_label.pack(pady=10)

progress_bar = ttk.Progressbar(
    root, orient="horizontal", length=700, mode="determinate"
)
progress_bar.pack(pady=10)

time_label = tk.Label(
    root,
    text="Time Remaining: Calculating...",
    font=("Arial", 12),
    bg="black",
    fg="white",
)
time_label.pack(pady=10)

hps_label = tk.Label(
    root,
    text="Hashes per Second: Calculating...",
    font=("Arial", 12),
    bg="black",
    fg="white",
)
hps_label.pack(pady=10)

# Result label and textbox
result_label = tk.Label(
    root, text="Result:", font=("Arial", 12), bg="black", fg="white"
)
result_label.pack(pady=10)

result_text = tk.Text(
    root, height=10, width=80, font=("Arial", 12), bg="gray10", fg="white"
)
result_text.pack(pady=10)

# Buttons
button_frame = tk.Frame(root, bg="black")
button_frame.pack(pady=20)

start_button = tk.Button(
    button_frame,
    text="Start Brute-Force",
    font=("Arial", 12),
    command=start_bruteforce,
)
start_button.grid(row=0, column=0, padx=10)

cancel_button = tk.Button(
    button_frame,
    text="Cancel",
    font=("Arial", 12),
    command=cancel_bruteforce,
)
cancel_button.grid(row=0, column=1, padx=10)

# Run the application
root.mainloop()