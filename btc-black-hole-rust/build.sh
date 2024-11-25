#!/bin/zsh

# Exit immediately if a command exits with a non-zero status
set -e

# Function to print messages in bold for better visibility
function print_message() {
    echo "\033[1m$1\033[0m"
}

# Step 1: Clean the project
print_message "Step 1: Cleaning the project..."
cargo clean
echo "✅ Project cleaned successfully.\n"

# Step 2: Build the project in release mode with optimizations
print_message "Step 2: Building the project (release mode)..."
export RUSTFLAGS="-C target-cpu=native -C target-feature=+neon"
cargo build --release
echo "✅ Project built successfully.\n"

# Step 3: Run unit tests
print_message "Step 3: Running unit tests..."
cargo test
echo "✅ All unit tests passed successfully.\n"

# Step 4: Execute the program
print_message "Step 4: Executing the program..."
./target/release/btc-black-hole-rust
echo "✅ Program executed successfully."