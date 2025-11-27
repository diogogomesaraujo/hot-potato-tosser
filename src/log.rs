//! Module that standardizes the logs displayed by peers and the server.

use color_print::cprintln;
use terminal_size::{terminal_size, Height};

/// Function that displays a log of type `Warning`.
pub fn warning(message: &str) {
    cprintln!("<yellow, bold>WARNING:</yellow, bold>  {}", message);
}

/// Function that displays a log of type `Information`.
pub fn info(message: &str) {
    cprintln!("<green, bold>INFO:</green, bold>     {}", message);
}

/// Function that displays a log of type `Debug`.
pub fn debug(message: &str) {
    cprintln!("<bold>DEBUG:</bold>    {}", message);
}

/// Function that displays a log of type `Error`.
pub fn error(message: &str) {
    cprintln!("<red, bold>ERROR:</red, bold>    {}", message);
}

/// Function that clears the terminal according to its height.
pub fn clear() {
    let size = terminal_size();
    if let Some((_, Height(h))) = size {
        for _ in 0..h {
            println!();
        }
    } else {
        for _ in 0..10 {
            println!();
        }
    }
}
