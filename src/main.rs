use terminal_size::{Width, Height, terminal_size};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{RawTerminal, IntoRawMode};
use termion::color;
use std::io::{Write, stdout, stdin, Stdin, Stdout};
use std::env;

// Differents mode for reading / editing
enum ViewModes
{
    COMMAND,
    INSERT
}

#[derive(Debug)]
enum Instr {
    LOAD_FILE,
    SAVE_FILE,
    QUIT,
    CHANGE_MODE_COMMAND,
    CHANGE_MODE_INSERT,
}

#[derive(Copy, Clone)]
struct Size2 {
    x : u16,
    y : u16
}

impl Size2 {
    pub fn new(x : u16, y : u16) -> Self {
        Self {
            x,
            y,
        }
    }
}

struct EscChr;

impl EscChr {
    const CLR : &str = "\x1Bc";
}

fn initScreen(stdout : &mut RawTerminal<Stdout>) -> Option<Size2> {
    // Empties the screen, sets the correct cursor
    write!(stdout, "{}{}{}", 
           EscChr::CLR,
           termion::cursor::BlinkingBlock,
           termion::cursor::Goto(1, 1)).unwrap();
    // Tries to get the terminal size
    if let Some((Width(w), Height(h))) = terminal_size() {
        // Prints a '~' at the begining of each line
        for i in 1..h-1 {
            print!("{}{}~{}",
                   color::Fg(color::Blue),
                   termion::cursor::Goto(1, i),
                   color::Fg(color::Reset));
        }
        // Creates the command line prompt
        let mut empty_line = String::with_capacity(w.into());
        for i in 0..w { empty_line.push(' '); }
        print!("{}{}{}{}",
               termion::cursor::Goto(1, h-1),
               color::Bg(color::LightBlack),
               empty_line,
               color::Bg(color::Reset));
        print!("{}", termion::cursor::Goto(1, h));
        // Flushes the output
        stdout.flush().unwrap();
        Some(Size2::new(w, h))
    } else {
        // Exits if cannot get terminal size
        println!("Unable to get term size, exiting tbe");
        None
    }
}

fn remove(stri : &mut String, n : u16) {
    for i in 0..n {
        stri.remove(0);
    }
}

fn execCmd(cmd : &mut String, term_s : &mut Size2) -> Vec<Instr> {
    let mut ret : Vec<Instr> = Vec::new();
    
    if cmd.starts_with(":") {
        cmd.remove(0);
        while !cmd.is_empty() {
            if cmd.starts_with("quit") {
                ret.push(Instr::QUIT);
                remove(cmd, 4);
            } else if cmd.starts_with("q") {
                ret.push(Instr::QUIT);
                remove(cmd, 1);
            } else {
                remove(cmd, 1);
            }
        }
    }

    ret
}

// Function that handles the INSERT mode, from switching back to COMMAND mode to just typing
fn insert(stdin : &mut Stdin, stdout : &mut RawTerminal<Stdout>, term_s : &mut Size2) -> Vec<Instr> {
    let mut ret : Vec<Instr> = Vec::new();

    for c in stdin.keys() {
        match c.unwrap() {
            // Escape key returns into COMMAND mode
            Key::Esc => {
                ret.push(Instr::CHANGE_MODE_COMMAND);
                break;
            },
            _ => {}
        }
    }
    stdout.flush().unwrap();
    ret
}

// Function that handles the COMMAND mode, manually collecting the command and handling the
// execution
fn command(stdin : &mut Stdin, stdout : &mut RawTerminal<Stdout>, term_s : &mut Size2) -> Vec<Instr> {
    let mut cmd : String = String::new();
    let mut curr_pos : Size2 = Size2::new(1, term_s.y);
    
    let mut ret : Vec<Instr> = Vec::new();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('\n') => {
                print!("{}{}",
                       termion::cursor::Goto(1, term_s.y),
                       termion::clear::CurrentLine);
                ret = execCmd(&mut cmd, term_s);
                break;
            },
            Key::Backspace => {
                if curr_pos.x > 0 {
                    cmd.pop();
                    print!("{}{}{}",termion::clear::CurrentLine,
                           termion::cursor::Goto(1, curr_pos.y),
                           cmd);
                    curr_pos.x -= 1;
                }
            },
            Key::Char(c) => {
                if c == 'i' && cmd.is_empty() {
                    ret.push(Instr::CHANGE_MODE_INSERT);
                    break;
                } else {
                    print!("{}", c);
                    cmd.push(c);
                }
            },
            _ => {
                curr_pos.x -= 1;
            }
        }
        curr_pos.x += 1;
        stdout.flush().unwrap();
    }
    stdout.flush().unwrap();
    ret
}

// Entry point
fn main() {
    // Gets the arguments given to the program through command prompt
    let args : Vec<String> = env::args().collect();
    let mut file_path : &String = &String::from(".");

    if args.len() != 0 {
        file_path = &args[0];
    }

    // Changes and sets the way stdin and stdout are handled
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut stdin = stdin();

    // Basic information needed for the program
    let mut run : bool = true;
    let mut term_size : Size2 = Size2::new(0, 0);

    // Tries to get the actual size of the terminal, exits the program if failed to
    if let Some(term_s) = initScreen(&mut stdout) {
        term_size = term_s;
    } else {
        run = false;
    }

    // Sets the launching mode to COMMAND
    let mut curr_mode : ViewModes = ViewModes::COMMAND;

    // Main loop
    while run {
        // Instructions given by the different handlers for the program to execute
        let mut instrs : Vec<Instr> = Vec::new();

        // Chooses which function to call depending on the current mode
        match curr_mode {
            // Gets a set of instructions back from the COMMAND mode handler
            ViewModes::COMMAND  => instrs = command(&mut stdin, &mut stdout, &mut term_size),
            ViewModes::INSERT   => instrs = insert(&mut stdin, &mut stdout, &mut term_size),
        }

        // Loops over all the instructions to be executed
        for instr in instrs {
            match instr {
                Instr::QUIT                 => run = false,
                Instr::CHANGE_MODE_COMMAND  => curr_mode = ViewModes::COMMAND,
                Instr::CHANGE_MODE_INSERT   => curr_mode = ViewModes::INSERT,
                _ => {},
            }
        }

    }

    // Exits the program, clears the screen
    print!("{}Exiting...", termion::cursor::Goto(1, 1));
    print!("{}{}{}",
           EscChr::CLR,
           color::Fg(color::Reset),
           color::Bg(color::Reset));
}
