# AudioPlayer

## Overview
AudioPlayer is a Rust-based application for playing audio files using the `cpal` and `symphonia` libraries. It supports loading and playing MP3 files, controlling volume, and managing playback through a command-based interface.

## Features
- Play audio files (e.g., MP3) with `symphonia` decoding.
- Adjust volume dynamically during playback.
- Asynchronous audio processing using `cpal`.
- Command-based control for play, stop, and volume adjustment.
- Thread-safe audio data management with `Arc` and `Mutex`.

## Requirements
- Rust (stable version).
- Supported audio file (e.g., MP3) for testing.
- Dependencies: `cpal`, `symphonia`, `std`.

## Installation
1. Clone the repository:
   ```bash
   git clone https://github.com/INFORGi/AudioPlayer.git
   cd AudioPlayer
   ```
2. Ensure Rust is installed: https://www.rust-lang.org/tools/install
3. Build and run the project:
   ```bash
   cargo run
   ```

## Usage
1. Place an MP3 file at a valid path (e.g., `C:/Users/shulg/Downloads/test.mp3`).
2. Run the application: `cargo run`.
3. The program will:
   - Load and play the audio file.
   - Adjust volume to 10% after 2 seconds.
   - Adjust volume to 30% after another 2 seconds.

## Dependencies
- `cpal` — for cross-platform audio playback.
- `symphonia` — for audio decoding.
- `std` — for threading, synchronization, and file operations.

## Notes
- Ensure the audio file path in `main.rs` is valid for your system.
- The application uses a hardcoded path for testing; modify as needed.
- Volume is controlled via a 0–1000 scale (e.g., 100 = 10%, 500 = 50%).

## License
MIT License
