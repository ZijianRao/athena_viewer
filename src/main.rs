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

    fn get_sliding_display_output(&self) -> String {
        // Calculate available lines in output area (excluding header, "Your input:", and separator area)
        const TERMINAL_HEIGHT: u16 = 24;
        const HEADER_HEIGHT: u16 = 6; // 5 header lines + "Your input:" label
        const SEPARATOR_HEIGHT: u16 = 3; // 2 separator lines + 1 input line
        const MAX_OUTPUT_LINES: usize = (TERMINAL_HEIGHT - HEADER_HEIGHT - SEPARATOR_HEIGHT) as usize;

        // Create a list of all lines to display (committed + current)
        let mut all_lines: Vec<String> = self.output_lines.clone();
        if !self.current_input.is_empty() {
            all_lines.push(self.current_input.clone());
        }

        // If we have more lines than can fit, take only the most recent ones
        if all_lines.len() > MAX_OUTPUT_LINES {
            let start_idx = all_lines.len() - MAX_OUTPUT_LINES;
            all_lines[start_idx..].join("\n")
        } else {
            all_lines.join("\n")
        }
    }

  }

fn render_echo_display(state: &EchoState) {
    // Clear screen and move to top
    print!("\x1b[2J\x1b[H");
    io::stdout().flush().unwrap();

    // Terminal layout constants
    const TOTAL_LINES: u16 = 24;

    // Header
    println!("=== Interactive Character Echo ===");
    println!("Type characters to see them echoed above instantly");
    println!("Press ESC or Ctrl+C to quit | Enter to submit line | Backspace to delete");
    println!("(Showing most recent {} lines)", state.max_lines);
    println!();

    // Calculate spacing for bottom-aligned display
    const TERMINAL_HEIGHT: u16 = 24;
    const HEADER_HEIGHT: u16 = 6; // 5 header lines + "Your input:" label
    const SEPARATOR_HEIGHT: u16 = 3; // 2 separator lines + 1 input line
    const MAX_OUTPUT_LINES: usize = (TERMINAL_HEIGHT - HEADER_HEIGHT - SEPARATOR_HEIGHT) as usize;

    // Get the sliding display output (most recent lines at bottom)
    let display_output = state.get_sliding_display_output();
    let display_lines: Vec<&str> = display_output.lines().collect();
    let num_display_lines = display_lines.len();

    println!("Your input:");

    // Calculate how many empty lines to add above to push content to bottom
    let empty_lines_above = MAX_OUTPUT_LINES.saturating_sub(num_display_lines);
    for _ in 0..empty_lines_above {
        println!();
    }

    // Display the content (oldest at top, newest at bottom of output area)
    if !display_output.is_empty() {
        for line in display_lines {
            println!("  {}", line);
        }
    } else {
        println!("  (start typing...)");
    }

    // Move to top separator line (above input)
    print!("\x1b[{};1H", TOTAL_LINES - 2);

    // Top separator line
    for _ in 0..80 {
        print!("═");
    }
    println!();

    // Position cursor at input line
    print!("\x1b[{};1H", TOTAL_LINES - 1);

    // Clear input line and show prompt
    print!("\x1b[K"); // Clear to end of line
    print!("❯ {}_", state.current_input);

    // Move cursor to correct position in input line
    print!("\x1b[{};{}H", TOTAL_LINES - 1, 4 + state.current_input.len());

    // Move to bottom separator line (below input)
    print!("\x1b[{};1H", TOTAL_LINES);

    // Bottom separator line
    for _ in 0..80 {
        print!("═");
    }

    // Move cursor back to input line
    print!("\x1b[{};{}H", TOTAL_LINES - 1, 4 + state.current_input.len());
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
                    let char_to_add = ch.chars().next().unwrap_or('\0');
                    state.add_char(char_to_add);
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

    // Restore terminal mode
    let _ = disable_raw_mode(fd, &original_termios);

    Ok(())
}
