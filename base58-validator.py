import tkinter as tk

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

def update_length(*args):
    """
    Update the string length label and validate the string whenever the entry text changes.
    Also highlight invalid characters in the string.
    """
    input_string = entry.get()
    length_label.config(text=f"Length: {len(input_string)} characters")
    
    # Validate the string and update the result
    if is_base58_valid(input_string):
        result_label.config(text="Valid Base58 ✔", fg="green")
        result_icon.config(text="✔", fg="green")  # Use checkmark as text
    else:
        result_label.config(text="Invalid Base58 ❌", fg="red")
        result_icon.config(text="❌", fg="red")  # Use crossmark as text
    
    # Update the highlighted text
    highlight_invalid_characters(input_string)

def highlight_invalid_characters(input_string):
    """
    Highlight invalid characters in bold and red, and valid characters in white.
    """
    text_widget.config(state=tk.NORMAL)  # Temporarily enable the Text widget
    text_widget.delete(1.0, tk.END)  # Clear the text widget
    
    for char in input_string:
        if char in BASE58_SET:
            # Valid characters are added in white
            text_widget.insert(tk.END, char, 'valid')
        else:
            # Invalid characters are inserted with bold and red color
            text_widget.insert(tk.END, char, 'invalid')
    
    # Re-disable the Text widget after updating
    text_widget.config(state=tk.DISABLED)

# Create the main window
root = tk.Tk()
root.title("Base58 Validator")
root.geometry("400x450")  # Set window size
root.configure(bg="black")  # Set background color for better contrast

# Create a label
label = tk.Label(root, text="Enter string to validate as Base58:", font=("Arial", 12), bg="black", fg="white")
label.pack(pady=10)

# Create an entry widget for input
entry = tk.Entry(root, font=("Arial", 12), width=40)
entry.pack(pady=10)

# Bind the update_length function to the entry field so it triggers on text change
entry.bind("<KeyRelease>", update_length)

# Create a label for displaying the string length
length_label = tk.Label(root, text="Length: 0 characters", font=("Arial", 10), bg="black", fg="white")
length_label.pack(pady=5)

# Create a label for displaying the result
result_label = tk.Label(root, text="", font=("Arial", 14), bg="black")
result_label.pack(pady=10)

# Create a label for displaying the icon
result_icon = tk.Label(root, font=("Arial", 20), bg="black")
result_icon.pack(pady=10)

# Create a Text widget for displaying the string with highlighted invalid characters
text_widget = tk.Text(root, height=2, width=40, font=('Arial', 12), bg="black")
text_widget.pack(pady=10)

# Configure tags for styling in the Text widget
text_widget.tag_configure('valid', foreground='white')  # White text for valid characters
text_widget.tag_configure('invalid', foreground='red', font=('Arial', 12, 'bold'))  # Red bold text for invalid characters
text_widget.config(state=tk.DISABLED)  # Initially disable the Text widget

# Run the application
root.mainloop()