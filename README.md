BTC Black Hole Brute-Force Tool
===============================

A Rust-based GUI application that brute-forces Bitcoin addresses to find valid ones based on a given base address. **Please note that any BTC sent to the addresses generated by this program will be irretrievably lost**, as the program does not generate corresponding public or private keys.

Table of Contents
-----------------

*   [Features](https://chatgpt.com/c/674244f8-ea1c-8005-a316-06fff09179ba#features)
    
*   [Disclaimer](https://chatgpt.com/c/674244f8-ea1c-8005-a316-06fff09179ba#disclaimer)
    
*   [Important Notice](https://chatgpt.com/c/674244f8-ea1c-8005-a316-06fff09179ba#important-notice)
    
*   [Prerequisites](https://chatgpt.com/c/674244f8-ea1c-8005-a316-06fff09179ba#prerequisites)
    
*   [Installation](https://chatgpt.com/c/674244f8-ea1c-8005-a316-06fff09179ba#installation)
    
*   [Usage](https://chatgpt.com/c/674244f8-ea1c-8005-a316-06fff09179ba#usage)
    
*   [Optimization for Apple Silicon M1](https://chatgpt.com/c/674244f8-ea1c-8005-a316-06fff09179ba#optimization-for-apple-silicon-m1)
    
*   [Project Structure](https://chatgpt.com/c/674244f8-ea1c-8005-a316-06fff09179ba#project-structure)
    
*   [Contributing](https://chatgpt.com/c/674244f8-ea1c-8005-a316-06fff09179ba#contributing)
    
*   [License](https://chatgpt.com/c/674244f8-ea1c-8005-a316-06fff09179ba#license)
    

Features
--------

*   **Multi-threaded Brute-Forcing**: Utilize all available CPU cores to maximize performance.
    
*   **Customizable Input**: Specify a base address and starting range in Base58.
    
*   **Real-time Progress Monitoring**: View thread information, progress bars, and hash rates.
    
*   **Optimized for M1**: Compiler optimizations for Apple Silicon processors.
    
*   **Graphical User Interface**: Built with eframe and egui for an intuitive GUI experience.
    

Disclaimer
----------

**Warning**: This tool is intended for educational and research purposes only. Brute-forcing Bitcoin addresses may be illegal or unethical in some jurisdictions. Use this tool responsibly and at your own risk. The authors are not responsible for any misuse or damages caused by this software.

Important Notice
----------------

**Do not send any Bitcoin (BTC) to the addresses generated by this program.** The program does not generate public or private keys for these addresses, making it impossible to access or retrieve any funds sent to them. Any BTC sent will be **lost forever**.

Prerequisites
-------------

*   **Rust**: Ensure you have Rust and Cargo installed. If not, install them from [rust-lang.org](https://www.rust-lang.org/tools/install).
    
*   **Cargo**: Comes bundled with Rust; used for building and running the project.
    

Installation
------------

1. **Clone the Repository**
```bash
    git clone https://github.com/yourusername/btc-black-hole-rust.git
    cd btc-black-hole-rust
```

2.  **Build and run the project**
```bash
    ./build.sh
```

Usage
-----

1. **This will launch the GUI application**
```bash
    cargo run --release
```

2.  **Using the Application**
    
*   **Base58 Address** Enter the base Bitcoin address without the checksum.
        
*   **Starting Range** (Optional) Specify a starting range in Base58.
        
*   **Number of Threads** Adjust the number of threads to use (defaults to the number of available CPU cores).
        
*   **Start Brute-Force** Click the "Start Brute-Force" button to begin.
        
3.  **Monitoring Progress**
    
*   **Thread Information** View each thread''s ID, start/end ranges, current candidate address, and remaining calculations.
        
*   **Progress Bar** Shows the overall progress of the brute-force operation.
        
*   **Total Hashes per Second** Displays the combined hash rate of all threads.
        
*   **Found Addresses** Lists any valid Bitcoin addresses found during the operation.
        
4.  **Canceling the Operation**
    
*   Click the "Cancel" button to stop the brute-force process at any time.
        

Optimization for Apple Silicon M1
---------------------------------

The project includes compiler optimizations tailored for Apple Silicon M1 processors:

*   -[Link Time Optimization] (LTO): Enabled for better performance.
    
*   -[Code Generation Units]: Set to 1 to optimize code generation.
    
*   -[Panic Strategy] Set to "abort" to reduce binary size and improve performance.
    
*   -[Optimization Level] Set to "3" in release profile for maximum optimization.
    

These settings are specified in the Cargo.toml file:

```bash
    [profile.release]
    opt-level = "3"
    lto = true
    codegen-units = 1
    panic = "abort"
```

Project Structure
-----------------

*   **src/main.rs**: Entry point of the application.
    
*   **src/lib.rs**: Core application logic and GUI implementation.
    
*   **Cargo.toml**: Project metadata and dependencies.
    
*   **tests/brute\_force\_tests.rs**: Unit tests for the application.
    

Contributing
------------

Contributions are welcome! Please follow these steps:

1.  Fork the repository.
    
2.  Create a new branch with a descriptive name.
    
3.  Make your changes and commit them with clear messages.
    
4.  Submit a pull request to the main branch.
    

License
-------

This project is licensed under the MIT License. See the [LICENSE](https://chatgpt.com/c/LICENSE) file for details.

**Note**:

*   Always ensure you're complying with local laws and regulations when using tools like this. Unauthorized access or attempts to access computer systems or networks without permission is illegal.