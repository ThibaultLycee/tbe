use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::{Write, stdout, stdin, Stdin, Stdout};
use std::io::prelude::*;
use std::process::Command;

use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{RawTerminal, IntoRawMode};

use terminal_size::{Width, Height, terminal_size};

// Differents mode for reading / editing
enum ViewModes
{
    COMMAND,
    INSERT,
    RUNNING_CMD,
}

enum Instr {
    LOAD_FILE(String),
    SAVE_FILE,
    QUIT,
    RUN(String),
    CHANGE_MODE_COMMAND,
    CHANGE_MODE_INSERT,
    CHANGE_MODE_RUNNING_CMD,
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

fn clearSeparatorLine(term_s : &mut Size2) {
    let mut empty_line = String::with_capacity(term_s.x.into());
    for i in 0..term_s.x { empty_line.push(' '); }
    print!("{}{}{}{}",
           termion::cursor::Goto(1, term_s.y-1),
           color::Bg(color::LightBlack),
           empty_line,
           color::Bg(color::Reset));
}

fn setupScreen(stdout : &mut RawTerminal<Stdout>, term_s : &mut Size2) {
    write!(stdout, "{}{}",
           EscChr::CLR,
           termion::cursor::Goto(1, 1)).unwrap();
    clearSeparatorLine(term_s);
    for i in 1..term_s.y-1 {
        print!("{}{}~{}",
               color::Fg(color::Blue),
               termion::cursor::Goto(1, i),
               color::Fg(color::Reset));
    }
}

fn initScreen(stdout : &mut RawTerminal<Stdout>) -> Option<Size2> {
    // Tries to get the terminal size
    if let Some((Width(w), Height(h))) = terminal_size() {
        // Prints a '~' at the begining of each line
        setupScreen(stdout, &mut Size2::new(w, h));
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

fn loadFile(path : &mut String, buff : &mut Vec<String>) -> std::io::Result<()> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    for line in contents.split('\n')
    {
        buff.push(String::from(line));
    }

    Ok(())
}

fn updateBuffer(buffer : &mut Vec<String>, coord : &mut Size2, offset : &mut Size2, char : &str) {
    //let off_y = (offset.y + coord.y) as u16;
    buffer[(offset.y + coord.y) as usize].replace_range(coord.x as usize..(coord.x+1) as usize, char);
}

fn showEntireBuffer(buffer : &mut Vec<String>, offset : &mut Size2, stdout : &mut RawTerminal<Stdout>, term_s : &mut Size2) {
    let mut i = 0;
    while i < term_s.y-2 && usize::from(i + offset.y) < buffer.len() {
        let line_nbr : u16 = offset.y + i;
        let line : &String = &buffer[line_nbr as usize];
        print!("{}{}{}{} {}",
               termion::cursor::Goto(1, i+1),
               color::Fg(color::LightYellow),
               line_nbr + 1,
               color::Fg(color::Reset),
               line);
        i += 1;
    }
    stdout.flush();
}

fn remove(stri : &mut String, n : u16) {
    for i in 0..n {
        stri.remove(0);
    }
}

fn execCmd(cmd : &mut String, term_s : &mut Size2) -> Vec<Instr> {
    let mut ret : Vec<Instr> = Vec::new();
    clearSeparatorLine(term_s);
    print!("{}", termion::cursor::Goto(1, term_s.y));

    if cmd.starts_with(":") {
        cmd.remove(0);
        while !cmd.is_empty() {
            if cmd.starts_with("quit") {
                ret.push(Instr::QUIT);
                remove(cmd, 4);
            } else if cmd.starts_with("run") {
                remove(cmd, 3);
                ret.push(Instr::CHANGE_MODE_RUNNING_CMD);
                ret.push(Instr::RUN(cmd.to_string()));
                break;
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

// Function that handles the COMMAND mode, manually collecting the command and handling the execution
fn command(stdin : &mut Stdin, stdout : &mut RawTerminal<Stdout>, term_s : &mut Size2) -> Vec<Instr> {
    print!("{}", termion::cursor::Goto(1, term_s.y));
    stdout.flush();
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

fn runningCommand(stdin : &mut Stdin, stdout : &mut RawTerminal<Stdout>, term_s : &mut Size2, cmd : &mut String) -> Vec<Instr> {
    let mut ret : Vec<Instr> = Vec::new();
    setupScreen(stdout, term_s);
    print!("{}{}Running : {}{}",
           termion::cursor::Goto(1, 1),
           color::Fg(color::Green),
           cmd,
           color::Fg(color::Reset));

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("failed to execute process")
    };

    let output_iter = std::str::from_utf8(&output.stdout).unwrap().split("\n");

    let mut h = 2;
    for line in output_iter {
        print!("{} {}",
               termion::cursor::Goto(1, h),
               line);
        h += 1;
        if h > term_s.y - 3 { break; }
    }

    print!("{}{}{}{}{}",
           termion::cursor::Goto(1, term_s.y-1),
           color::Bg(color::LightBlack),
           output.status,
           color::Bg(color::Reset),
           termion::cursor::Goto(1, term_s.y));

    stdout.flush();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('\n') => {
                ret.push(Instr::CHANGE_MODE_COMMAND);
                break;
            },
            _ => {},
        }
    }
    ret
}

// Entry point
fn main() {
    // Gets the arguments given to the program through command prompt
    let args : Vec<String> = env::args().collect();
    let mut file_path : String = String::from(".");
    let mut buffer : Vec<String> = Vec::new();

    if args.len() != 0 {
        file_path = String::from(&args[0]);
        if loadFile(&mut file_path, &mut buffer).is_err() {
            buffer = Vec::new();
        }
    }

    // Debug purposes only
    buffer.push(String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit."));
    buffer.push(String::from("Ut euismod at purus sed vehicula. In laoreet lectus ligula, sed aliquam lectus pellentesque sit amet."));
    buffer.push(String::from("Aliquam tincidunt sit amet ipsum at semper."));
    buffer.push(String::from("Praesent at ante vel dui sollicitudin interdum. Praesent lacus ante,"));
    buffer.push(String::from("accumsan sed dui sed, porta viverra augue. Aliquam id. "));

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

    let mut cmd : String = String::new();
    // Main loop
    while run {
        // Instructions given by the different handlers for the program to execute
        let mut instrs : Vec<Instr> = Vec::new();

        showEntireBuffer(&mut buffer, &mut Size2::new(0, 0), &mut stdout, &mut term_size);

        // Chooses which function to call depending on the current mode
        match curr_mode {
            // Gets a set of instructions back from the COMMAND mode handler
            ViewModes::COMMAND      => instrs = command(&mut stdin, &mut stdout, &mut term_size),
            ViewModes::INSERT       => instrs = insert(&mut stdin, &mut stdout, &mut term_size),
            ViewModes::RUNNING_CMD  => instrs = runningCommand(&mut stdin, &mut stdout, &mut term_size, &mut cmd),
        }

        // Loops over all the instructions to be executed
        for instr in instrs {
            match instr {
                Instr::QUIT                     => run = false,
                Instr::RUN(c)                   => cmd = c,
                Instr::CHANGE_MODE_COMMAND      => curr_mode = ViewModes::COMMAND,
                Instr::CHANGE_MODE_INSERT       => curr_mode = ViewModes::INSERT,
                Instr::CHANGE_MODE_RUNNING_CMD  => curr_mode = ViewModes::RUNNING_CMD,
                Instr::LOAD_FILE(mut path) => {
                    buffer = Vec::new();
                    if loadFile(&mut path, &mut buffer).is_err() {
                        buffer = Vec::new();
                    }
                }
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
