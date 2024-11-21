import tkinter as tk

# Define the Base58 alphabet
BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
BASE58_SET = set(BASE58_ALPHABET)

def highlight_invalid_characters(input_string):
    """
    Highlight invalid characters in bold and red, and valid characters in white.
    """
    result_text.config(state=tk.NORMAL)  # Temporarily enable the result Text widget
    result_text.delete(1.0, tk.END)  # Clear the result Text widget

    for line in input_string.splitlines():
        for char in line:
            if char in BASE58_SET:
                # Valid characters are added in white
                result_text.insert(tk.END, char, 'valid')
            else:
                # Invalid characters are inserted with bold and red color
                result_text.insert(tk.END, char, 'invalid')
        result_text.insert(tk.END, "\n")  # Add a newline after each address

    result_text.config(state=tk.DISABLED)  # Re-disable the result Text widget

def update_result(*args):
    """
    Update the result Text widget whenever the input changes.
    """
    input_text = input_textbox.get(1.0, tk.END).strip()  # Get text from the input Text widget
    highlight_invalid_characters(input_text)

# Create the main window
root = tk.Tk()
root.title("Bulk Base58 Validator")
root.geometry("800x450")  # Set window size
root.configure(bg="black")  # Set background color for better contrast

# Labels for the textboxes
input_label = tk.Label(root, text="Enter Base58 Addresses:", font=("Arial", 12), bg="black", fg="white")
input_label.grid(row=0, column=0, padx=10, pady=10, sticky="w")
result_label = tk.Label(root, text="Validation Results:", font=("Arial", 12), bg="black", fg="white")
result_label.grid(row=0, column=1, padx=10, pady=10, sticky="w")

# Input Text widget for entering Base58 addresses
input_textbox = tk.Text(root, height=20, width=40, font=('Arial', 12), bg="black", fg="white", insertbackground="white")
input_textbox.grid(row=1, column=0, padx=10, pady=10)
input_textbox.bind("<KeyRelease>", update_result)  # Trigger result update on text change

# Result Text widget for displaying validation results
result_text = tk.Text(root, height=20, width=40, font=('Arial', 12), bg="black")
result_text.grid(row=1, column=1, padx=10, pady=10)

# Configure tags for styling in the Result Text widget
result_text.tag_configure('valid', foreground='white')  # White text for valid characters
result_text.tag_configure('invalid', foreground='red', font=('Arial', 12, 'bold'))  # Red bold text for invalid characters
result_text.config(state=tk.DISABLED)  # Initially disable the Result Text widget

# Run the application
root.mainloop()