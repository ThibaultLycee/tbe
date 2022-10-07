use terminal_size::{Width, Height, terminal_size};

// Differents mode for reading / editing
enum ViewModes
{
    COMMAND,
    INSERT
}

fn clearTerminal() {
    print!("\x1Bc");
}

fn moveCursor(x : u16, y : u16) {
    print!("\x1B[{};{}H", y, x);
}

fn initScreen() {
    clearTerminal();
    moveCursor(1, 1);
    if let Some((Width(w), Height(h))) = terminal_size() {
        for i in 1..h {
            moveCursor(1, i);
            print!("~");
        }
    } else {
        println!("Unable to get term size, exiting tbe");
    }
}

fn main() {
    initScreen();
    println!("Test");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
}
