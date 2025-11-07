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
    input: String,
}

impl EchoState {
    fn new() -> Self {
        Self {
            input: String::new(),
        }
    }

    fn add_char(&mut self, ch: char) {
        self.input.push(ch);
    }

    fn remove_char(&mut self) {
        self.input.pop();
    }
}

fn render_echo_display(state: &EchoState) {
    // Clear screen and move to top
    print!("\x1b[2J\x1b[H");
    io::stdout().flush().unwrap();

    // Header
    println!("=== Interactive Character Echo ===");
    println!("Type characters to see them echoed above instantly");
    println!("Press ESC or Ctrl+C to quit | Backspace to delete");
    println!();

    // Display area
    println!("Your input:");
    println!("  {}", state.input);
    println!();

    // Input prompt
    print!("❯ {}_", state.input);
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
    println!("You typed: \"{}\"", state.input);

    Ok(())
}
