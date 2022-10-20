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
    if let Some((w, h)) = termion::terminal_size().ok() {
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
    print!("test");
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    for line in contents.split('\n')
    {
        buff.push(String::from(line));
    }

    buff.pop();

    Ok(())
}

fn saveFile(path : &mut String, buff : &mut Vec<String>) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    print!("{}test", termion::cursor::Goto(1, 50));
    for line in buff {
        file.write_all(&line.as_bytes().to_vec());
        file.write_all(&[10]);
    }
    Ok(())
}

fn updateBuffer(buffer : &mut Vec<String>, coord : &mut Size2, offset : &mut Size2, chr : char) {
    let mut line = &mut buffer[(offset.y + coord.y - 1) as usize];
    if line.len() > coord.x.into() {
        line.insert((coord.x - 5) as usize, chr);
    } else {
        line.push(chr);
    }
}

fn showBufferLine(buffer : &mut Vec<String>, offset : &mut Size2, stdout : &mut RawTerminal<Stdout>, term_s : &mut Size2, line : u16) {
    print!("{}{}{}{:>3}{} {}",
           termion::cursor::Goto(1, offset.y),
           termion::clear::CurrentLine,
           color::Fg(color::LightYellow),
           line+1,
           color::Fg(color::Reset),
           buffer[line as usize]);
}
    
fn showEntireBuffer(buffer : &mut Vec<String>, offset : &mut Size2, stdout : &mut RawTerminal<Stdout>, term_s : &mut Size2) {
    setupScreen(stdout, term_s);
    let mut i = 0;
    while i < term_s.y-2 && usize::from(i + offset.y) < buffer.len() {
        let line_nbr : u16 = offset.y + i;
        let line : &String = &buffer[line_nbr as usize];
    
        print!("{}{}{:>3}{} {}",
               termion::cursor::Goto(1, i+1),
               color::Fg(color::LightYellow),
               line_nbr+1,
               color::Fg(color::Reset),
               line);

        i += 1;
    }
    stdout.flush();
}

fn getTermSize() -> std::result::Result<Size2, String> {
    if let (x, y) = termion::terminal_size().unwrap() {
        Ok(Size2::new(x, y))
    } else {
        Err(String::from("failed to get terminal size"))
    }
}

fn remove(stri : &mut String, n : u16) {
    for i in 0..n {
        stri.remove(0);
    }
}

fn trim(stri : &mut String) {
    while stri.starts_with(" ") {
        remove(stri, 1);
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
                trim(cmd);
                ret.push(Instr::CHANGE_MODE_RUNNING_CMD);
                ret.push(Instr::RUN(cmd.to_string()));
                break;
            } else if cmd.starts_with("q") {
                ret.push(Instr::QUIT);
                remove(cmd, 1);
            } else if cmd.starts_with("w") {
                ret.push(Instr::SAVE_FILE);
                remove(cmd, 1);
            } else if cmd.starts_with("o") {
                remove(cmd, 1);
                trim(cmd);
                ret.push(Instr::LOAD_FILE(cmd.to_string()));
                break;
            } else {
                remove(cmd, 1);
            }
        }
    }
    ret
}

// Function that handles the INSERT mode, from switching back to COMMAND mode to just typing
fn insert(stdin : &mut Stdin, stdout : &mut RawTerminal<Stdout>, term_s : &mut Size2, coord : &mut Size2, buffer : &mut Vec<String>) -> Vec<Instr> {
    let mut ret : Vec<Instr> = Vec::new();
    
    print!("{}", termion::cursor::Goto(coord.x+4, coord.y));
    stdout.flush().unwrap();

    for c in stdin.keys() {
        match c.unwrap() {
            // Escape key returns into COMMAND mode
            Key::Esc => {
                ret.push(Instr::CHANGE_MODE_COMMAND);
                break;
            },
            Key::Left => {
                if coord.x > 1 {
                    coord.x -= 1;
                }
            },
            Key::Right => {
                if (coord.x as usize) < buffer[coord.y as usize - 1].len() + 1{
                    coord.x += 1;
                }
            },
            Key::Up => {
                if coord.y > 1 {
                    coord.y -= 1;
                }
            },
            Key::Down => {
                if (coord.y as usize) < buffer.len() {
                    coord.y += 1;
                }
            },
            Key::Char('\n') => {
                let line = coord.y as usize - 1;
                let x = coord.x as usize - 1;
                let old_line = String::from(buffer[line].as_str());
                buffer.remove(line);
                buffer.insert(line, (old_line.as_str()[..x]).to_string());
                buffer.insert(line + 1, (old_line.as_str()[x..]).to_string());
                coord.y += 1;
                coord.x = 1;
                showEntireBuffer(buffer, &mut Size2::new(coord.x+4, 0), stdout, term_s);
            },
            Key::Delete => {
                coord.x = 1;
                buffer[coord.y as usize - 1] = String::new();
            },
            Key::Backspace => {
                let line = &mut buffer[coord.y as usize - 1];
                if coord.x > 1 {
                    line.remove(coord.x as usize - 2);
                    coord.x -= 1;
                } else {
                    // Merge the previous line with the current one
                    let line = coord.y as usize - 1;
                    if line > 0 {
                        let x = buffer[line - 1].len() + 1;
                        let line_content = String::from(buffer[line].as_str());
                        buffer[line - 1].push_str(line_content.as_str());
                        buffer.remove(line);
                        coord.y -= 1;
                        coord.x = x as u16;
                        showEntireBuffer(buffer, &mut Size2::new(coord.x + 4, 0), stdout, term_s);
                    }
                }
            },
            Key::Char(c) => {
                updateBuffer(buffer, &mut Size2::new(coord.x+4, coord.y), &mut Size2::new(0, 0), c);
                stdout.flush().unwrap();
                coord.x += 1;
            },
            _ => {}
        }
        // Puts back the pointer at the end of the current line if it exceeds it
        if (coord.x as usize) > buffer[coord.y as usize - 1].len() {
            coord.x = buffer[coord.y as usize - 1].len() as u16 + 1;
        }

        // Refreshes the screen
        showBufferLine(buffer, &mut Size2::new(coord.x+4, coord.y), stdout, term_s, coord.y-1);
        print!("{}", termion::cursor::Goto(coord.x+4, coord.y));
        stdout.flush().unwrap();
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
    let mut file_path : String = String::from("./temp.txt");
    let mut buffer : Vec<String> = Vec::new();

    if args.len() > 1 {
        file_path = String::from(&args[1]);
        if loadFile(&mut file_path, &mut buffer).is_err() {
            buffer = Vec::new();
            buffer.push(String::new());
        }
    } else {
        buffer.push(String::new());
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
    
    let mut msg : String = String::new();
    let mut cmd : String = String::new();
    let mut editing_coord : Size2 = Size2::new(1, 1);

    // Main loop
    while run {
        // Instructions given by the different handlers for the program to execute
        let mut instrs : Vec<Instr> = Vec::new();

        // Updates the screen, redraws everything
        showEntireBuffer(&mut buffer, &mut Size2::new(0, 0), &mut stdout, &mut term_size);
        clearSeparatorLine(&mut term_size);
        print!("{}{}{}{}",
               termion::cursor::Goto(1, term_size.y-1),
               color::Bg(color::LightBlack),
               msg,
               color::Bg(color::Reset));
        msg = String::new();

        // Chooses which function to call depending on the current mode
        match curr_mode {
            // Gets a set of instructions back from the current mode handler
            ViewModes::COMMAND      => instrs = command(&mut stdin, &mut stdout, &mut term_size),
            ViewModes::INSERT       => instrs = insert(&mut stdin, &mut stdout, &mut term_size, &mut editing_coord, &mut buffer),
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
                Instr::LOAD_FILE(mut path)      => {
                    buffer = Vec::new();
                    if loadFile(&mut path, &mut buffer).is_err() {
                        buffer = Vec::new();
                        buffer.push(String::new());
                        msg = format!("Failed to load file at {}", path);
                    } else {
                        file_path = path;
                        editing_coord = Size2::new(1, 1);
                        msg = format!("Oppened {}", file_path);
                    }
                },
                Instr::SAVE_FILE                => {
                    if saveFile(&mut file_path, &mut buffer).is_err() {
                        msg = format!("Failed to save file at {}", file_path);
                    } else {
                        msg = format!("Saved to {}", file_path);
                    }
                },
                _ => {},
            }
        }

        // Updates the terminal size, in case it has changed
        if let term_s = getTermSize().unwrap() {
            term_size = term_s;
        } else {
            run = false;
        }

    }

    // Exits the program, clears the screen
    print!("{}{}{}",
           EscChr::CLR,
           color::Fg(color::Reset),
           color::Bg(color::Reset));
}
