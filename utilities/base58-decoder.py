import tkinter as tk
import base58
from binascii import Error

# Test case 1BitcoinEaterAddressDontSendf59kuE

def base58_to_hex(base58_string):
    """
    Convert a Base58-encoded string to a hexadecimal representation.
    
    Parameters:
    base58_string (str): The Base58 string to decode.
    
    Returns:
    str: The hexadecimal representation, or an error message if decoding fails.
    """
    try:
        decoded_bytes = base58.b58decode(base58_string.strip())
        return decoded_bytes.hex()
    except (ValueError, Error):
        return "Invalid Base58"

def update_result(*args):
    """
    Update the result Text widgets whenever the input changes.
    """
    input_text = input_textbox.get(1.0, tk.END).strip()  # Get text from the input Text widget
    hex_text.config(state=tk.NORMAL)  # Temporarily enable the hex Text widget
    length_text.config(state=tk.NORMAL)  # Temporarily enable the length Text widget
    hex_text.delete(1.0, tk.END)  # Clear the hex Text widget
    length_text.delete(1.0, tk.END)  # Clear the length Text widget

    for line in input_text.splitlines():
        if line.strip():  # Skip empty lines
            hex_result = base58_to_hex(line)
            string_length = len(line.strip())
            hex_text.insert(tk.END, f"{hex_result}\n")
            length_text.insert(tk.END, f"{string_length}\n")
        else:
            hex_text.insert(tk.END, "\n")  # Preserve empty lines
            length_text.insert(tk.END, "\n")
    
    hex_text.config(state=tk.DISABLED)  # Re-disable the hex Text widget
    length_text.config(state=tk.DISABLED)  # Re-disable the length Text widget

# Create the main window
root = tk.Tk()
root.title("Base58 to Hex Decoder")
root.geometry("1200x450")  # Set window size
root.configure(bg="black")  # Set background color for better contrast

# Labels for the textboxes
input_label = tk.Label(root, text="Enter Base58 Addresses:", font=("Arial", 12), bg="black", fg="white")
input_label.grid(row=0, column=0, padx=10, pady=10, sticky="w")
hex_label = tk.Label(root, text="Decoded Hex Values:", font=("Arial", 12), bg="black", fg="white")
hex_label.grid(row=0, column=1, padx=10, pady=10, sticky="w")
length_label = tk.Label(root, text="String Length:", font=("Arial", 12), bg="black", fg="white")
length_label.grid(row=0, column=2, padx=10, pady=10, sticky="w")

# Input Text widget for entering Base58 addresses
input_textbox = tk.Text(root, height=20, width=40, font=('Arial', 12), bg="black", fg="white", insertbackground="white")
input_textbox.grid(row=1, column=0, padx=10, pady=10)
input_textbox.bind("<KeyRelease>", update_result)  # Trigger result update on text change

# Hex Text widget for displaying decoded HEX values
hex_text = tk.Text(root, height=20, width=60, font=('Arial', 12), bg="black", fg="white")
hex_text.grid(row=1, column=1, padx=10, pady=10)
hex_text.config(state=tk.DISABLED)  # Initially disable the Hex Text widget

# Length Text widget for displaying string lengths
length_text = tk.Text(root, height=20, width=20, font=('Arial', 12), bg="black", fg="white")
length_text.grid(row=1, column=2, padx=10, pady=10)
length_text.config(state=tk.DISABLED)  # Initially disable the Length Text widget

# Run the application
root.mainloop()