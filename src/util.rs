use std::io::{self, Write};

pub fn clear_console() {
    print!("\x1B[2J\x1B[3J\x1B[H");
    io::stdout().flush().unwrap();
}
