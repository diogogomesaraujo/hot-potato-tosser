use color_print::cprintln;
use terminal_size::{terminal_size, Height};

pub fn warning(message: &str) {
    cprintln!("<yellow, bold>WARNING:</yellow, bold>  {}", message);
}

pub fn info(message: &str) {
    cprintln!("<green, bold>INFO:</green, bold>     {}", message);
}

pub fn debug(message: &str) {
    cprintln!("<bold>DEBUG:</bold>    {}", message);
}

pub fn error(message: &str) {
    cprintln!("<red, bold>ERROR:</red, bold>    {}", message);
}

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
