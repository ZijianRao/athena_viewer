use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use libc::{tcgetattr, tcsetattr, termios, ECHO, ICANON, TCSANOW, VMIN, VTIME};

fn enable_raw_mode() -> io::Result<termios> {
    let fd = io::stdin().as_raw_fd();
    let mut original_termios: termios = unsafe { std::mem::zeroed() };

    unsafe {
        if tcgetattr(fd, &mut original_termios) != 0 {
            return Err(io::Error::last_os_error());
        }

        let mut raw = original_termios;
        raw.c_lflag &= !(ICANON | ECHO); // Disable canonical mode and echo
        raw.c_cc[VMIN] = 1;   // Wait for at least 1 character
        raw.c_cc[VTIME] = 0;  // No timeout (immediate)

        if tcsetattr(fd, TCSANOW, &raw) != 0 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(original_termios)
}

fn disable_raw_mode(fd: i32, original_termios: &termios) -> io::Result<()> {
    unsafe {
        if tcsetattr(fd, TCSANOW, original_termios) != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

struct EchoState {
    current_input: String,
    output_lines: Vec<String>,
    max_lines: usize,
}

impl EchoState {
    fn new() -> Self {
        Self {
            current_input: String::new(),
            output_lines: Vec::new(),
            max_lines: 5,
        }
    }

    fn add_char(&mut self, ch: char) {
        self.current_input.push(ch);
    }

    fn remove_char(&mut self) {
        self.current_input.pop();
    }

    fn submit_current_input(&mut self) {
        if !self.current_input.is_empty() {
            self.output_lines.push(self.current_input.clone());
            self.current_input.clear();

            // Keep only the most recent max_lines
            if self.output_lines.len() > self.max_lines {
                self.output_lines.remove(0);
            }
        }
    }

  }

fn render_echo_display(state: &EchoState) {
    // Clear screen and move to top
    print!("\x1b[2J\x1b[H");
    io::stdout().flush().unwrap();

    // Get terminal dimensions (assuming at least 24 lines)
    const TOTAL_LINES: u16 = 24;
    const INPUT_HEIGHT: u16 = 3; // Input line + prompt + separator
    const OUTPUT_HEIGHT: u16 = TOTAL_LINES - INPUT_HEIGHT;

    // Header
    println!("=== Interactive Character Echo ===");
    println!("Type characters to see them echoed above instantly");
    println!("Press ESC or Ctrl+C to quit | Enter to submit line | Backspace to delete");
    println!("(Showing most recent {} lines)", state.max_lines);
    println!();

    // Output area with recent history
    println!("Your input:");
    if !state.output_lines.is_empty() {
        for line in &state.output_lines {
            println!("  {}", line);
        }
    } else {
        println!("  (start typing...)");
    }

    // Fill remaining output area with empty lines
    let used_lines = 5 + state.output_lines.len(); // 4 header + input + lines
    for _ in 0..(OUTPUT_HEIGHT as usize - used_lines) {
        println!();
    }

    // Separator line
    for _ in 0..80 {
        print!("─");
    }
    println!();

    // Fixed input area at bottom (move to bottom, then up for input)
    print!("\x1b[{}A\x1b[G", INPUT_HEIGHT - 1);

    // Input prompt (always show current input line)
    print!("❯ {}_", state.current_input);

    // Move cursor to end of input
    print!("\x1b[{}C", state.current_input.len() + 3);
    io::stdout().flush().unwrap();
}

fn main() -> io::Result<()> {
    // Enable raw terminal mode for immediate character input
    let fd = io::stdin().as_raw_fd();
    let original_termios = match enable_raw_mode() {
        Ok(termios) => termios,
        Err(e) => {
            eprintln!("Error: Could not enable raw terminal mode: {}", e);
            eprintln!("This application requires raw terminal mode to function properly.");
            eprintln!("Please run in a proper terminal environment.");
            std::process::exit(1);
        }
    };

    let mut state = EchoState::new();
    let (tx, rx) = mpsc::channel();

    // Spawn input thread for raw character reading
    let input_thread = thread::spawn(move || {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = [0u8; 1];

        loop {
            match handle.read(&mut buffer) {
                Ok(1) => {
                    let ch = buffer[0] as char;

                    match ch {
                        '\x03' => { // Ctrl+C
                            let _ = tx.send(String::from("QUIT"));
                            break;
                        }
                        '\x1b' => { // ESC
                            // Try to read potential escape sequence, but treat as ESC
                            let _ = tx.send(String::from("QUIT"));
                            break;
                        }
                        '\r' | '\n' => { // Enter
                            let _ = tx.send(String::from("ENTER"));
                        }
                        '\x7f' | '\x08' => { // Backspace
                            let _ = tx.send(String::from("BACKSPACE"));
                        }
                        _ => {
                            // Regular character
                            let _ = tx.send(format!("CHAR:{}", ch));
                        }
                    }
                }
                Ok(0) => { // EOF
                    let _ = tx.send(String::from("QUIT"));
                    break;
                }
                Err(_) => {
                    // Continue trying
                    continue;
                }
                _ => {
                    // Read more bytes if available
                    continue;
                }
            }
        }
    });

    // Initial render
    render_echo_display(&state);

    // Main event loop
    loop {
        match rx.try_recv() {
            Ok(command) => {
                if command == "QUIT" {
                    break;
                } else if command == "ENTER" {
                    state.submit_current_input();
                    render_echo_display(&state);
                } else if command == "BACKSPACE" {
                    state.remove_char();
                    render_echo_display(&state);
                } else if command.starts_with("CHAR:") {
                    let ch = &command[5..];
                    state.add_char(ch.chars().next().unwrap_or('\0'));
                    render_echo_display(&state);
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                break;
            }
            Err(mpsc::TryRecvError::Empty) => {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    let _ = input_thread.join();

    // Restore terminal mode and clean up
    let _ = disable_raw_mode(fd, &original_termios);

    print!("\x1b[2J\x1b[H");
    io::stdout().flush().unwrap();
    println!("Goodbye!");

    if !state.output_lines.is_empty() || !state.current_input.is_empty() {
        println!("You typed:");
        for line in &state.output_lines {
            println!("  {}", line);
        }
        if !state.current_input.is_empty() {
            println!("  {}", state.current_input);
        }
    } else {
        println!("You typed: (nothing)");
    }

    Ok(())
}
