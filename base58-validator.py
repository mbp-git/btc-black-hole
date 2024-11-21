import tkinter as tk
import string

# Test case 1BitcoinEaterAddressDontSendf59kuE

# Define the Base58 alphabet
BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
BASE58_SET = set(BASE58_ALPHABET)

def is_base58_valid(input_string):
    """
    Check if a string is valid Base58.
    
    Parameters:
    input_string (str): The string to check.
    
    Returns:
    bool: True if the string is a valid Base58, False otherwise.
    """
    return all(char in BASE58_SET for char in input_string)

def check_base58():
    """
    Event handler for the "Check" button click.
    Validates the input and updates the interface with a check or cross icon.
    """
    input_string = entry.get()
    
    if is_base58_valid(input_string):
        result_label.config(text="Valid Base58 ✔", fg="green")
        result_icon.config(text="✔", fg="green")  # Use checkmark as text
    else:
        result_label.config(text="Invalid Base58 ❌", fg="red")
        result_icon.config(text="❌", fg="red")  # Use crossmark as text

# Create the main window
root = tk.Tk()
root.title("Base58 Validator")
root.geometry("400x300")  # Set window size

# Create a label
label = tk.Label(root, text="Enter string to validate as Base58:", font=("Arial", 12))
label.pack(pady=10)

# Create an entry widget for input
entry = tk.Entry(root, font=("Arial", 12), width=40)
entry.pack(pady=10)

# Create a button to trigger validation
check_button = tk.Button(root, text="Check Base58", font=("Arial", 12), command=check_base58)
check_button.pack(pady=20)

# Create a label for displaying the result
result_label = tk.Label(root, text="", font=("Arial", 14))
result_label.pack(pady=10)

# Create a label for displaying the icon
result_icon = tk.Label(root, font=("Arial", 20))
result_icon.pack(pady=10)

# Run the application
root.mainloop()