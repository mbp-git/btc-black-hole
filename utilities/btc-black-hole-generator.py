import tkinter as tk
from tkinter import ttk
from hashlib import sha256
import base58
import binascii
from tqdm import tqdm
from threading import Thread, Event

# Test case 1BitcoinEaterAddressDontSendf5????

def decode_base58_ignore_question(base58_string):
    """
    Decode a Base58 string, ignoring '?' characters.
    """
    filtered_string = base58_string.replace("?", "")
    try:
        decoded_bytes = base58.b58decode(filtered_string.strip())
        return decoded_bytes.hex()
    except (ValueError, binascii.Error):
        return "Invalid Base58"

def sha256d(data):
    """
    Perform double SHA-256 hashing.
    """
    return sha256(sha256(data).digest()).digest()

def brute_force_checksum(hex_payload, progress_label, progress_bar, stop_event):
    """
    Brute-force a valid checksum for the given payload.
    """
    payload = bytes.fromhex(hex_payload)
    if len(payload) != 21:
        result_text.insert(tk.END, "Invalid payload length. Expected 21 bytes.\n")
        return

    for i in tqdm(range(0x00000000, 0xFFFFFFFF + 1), desc="Brute-forcing checksum", ascii=True):
        if stop_event.is_set():  # Check if cancel is triggered
            result_text.insert(tk.END, "Brute-forcing cancelled.\n")
            return

        # Append the brute-forced checksum to the payload
        checksum = i.to_bytes(4, byteorder="big")
        test_payload = payload + checksum

        # Validate checksum
        if sha256d(test_payload[:-4])[:4] == test_payload[-4:]:
            valid_checksum = checksum.hex()
            result_text.insert(tk.END, f"Valid checksum found: {valid_checksum}\n")
            progress_label.config(text="Progress: Done!")
            progress_bar["value"] = 100
            return

        # Update progress in UI
        if i % 10000 == 0:  # Reduce UI updates for better performance
            progress_label.config(text=f"Progress: {i:#x}")
            progress_bar["value"] = (i / 0xFFFFFFFF) * 100
            progress_bar.update()

    result_text.insert(tk.END, "No valid checksum found.\n")
    progress_label.config(text="Progress: Finished.")

def start_bruteforce():
    """
    Start the brute-forcing process in a separate thread.
    """
    global stop_event
    stop_event.clear()  # Reset the stop event

    base58_input = input_textbox.get("1.0", tk.END).strip()
    hex_payload = decode_base58_ignore_question(base58_input)

    if hex_payload == "Invalid Base58":
        result_text.insert(tk.END, "Invalid Base58 string.\n")
        return

    if hex_payload.startswith("00"):
        hex_payload = hex_payload[2:]  # Remove version byte (00)

    payload = hex_payload[:42]  # Extract first 21 bytes (42 hex chars)
    progress_label.config(text="Starting brute force...")
    progress_bar["value"] = 0

    # Run brute-force in a separate thread
    thread = Thread(target=brute_force_checksum, args=(payload, progress_label, progress_bar, stop_event))
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
root.geometry("800x600")
root.configure(bg="black")

# Input label and textbox
input_label = tk.Label(root, text="Enter Base58 Address:", font=("Arial", 12), bg="black", fg="white")
input_label.pack(pady=10)

input_textbox = tk.Text(root, height=2, width=60, font=("Arial", 12), bg="black", fg="white", insertbackground="white")
input_textbox.pack(pady=10)

# Progress bar
progress_label = tk.Label(root, text="Progress: 0%", font=("Arial", 12), bg="black", fg="white")
progress_label.pack(pady=10)

progress_bar = ttk.Progressbar(root, orient="horizontal", length=700, mode="determinate")
progress_bar.pack(pady=10)

# Result label and textbox
result_label = tk.Label(root, text="Result:", font=("Arial", 12), bg="black", fg="white")
result_label.pack(pady=10)

result_text = tk.Text(root, height=10, width=80, font=("Arial", 12), bg="black", fg="white")
result_text.pack(pady=10)

# Buttons
button_frame = tk.Frame(root, bg="black")
button_frame.pack(pady=20)

start_button = tk.Button(button_frame, text="Start Brute-Force", font=("Arial", 12), command=start_bruteforce, bg="gray", fg="white")
start_button.grid(row=0, column=0, padx=10)

cancel_button = tk.Button(button_frame, text="Cancel", font=("Arial", 12), command=cancel_bruteforce, bg="gray", fg="white")
cancel_button.grid(row=0, column=1, padx=10)

# Run the application
root.mainloop()