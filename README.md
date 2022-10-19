# Text Based Editor

Text-based editor is a simple text editor written in [Rust](https://www.rust-lang.org)

## Inspiration

Tbe was tremandously inspided by [Vim](https://www.vim.org)

## Usage

For now, there are only a very restricted amount of working things, as it is still in devellopement, and only at it's beginning.

### Command mode

The command mode is the main mode for Tbe. As it is in Vim, it allows you to type in commands to quickly do stuff such as :
 - `:quit` or `:q` : used to quit Tbe
 - `:run "cmd"` : used to run a terminal command
 - `:o "file"` : used to open a file
 - `:w` : saves the file to the current open one, default to `temp.txt`

### Insert mode

The Insert mode allows you to edit the opened file by simply typing some stuff.
The use for different keys is as follow (checked if already implemented) :
 - [x] `chr` is used to type the choosen char
 - [x] `Backspace` to remove the previous char
 - [ ] `Enter` to print a new line
 - [x] `Delete` to delete the content of the current line

These are particular keys, used to do things not related to typing :
 - [x] `Esc` to get back to command mode

### Running Command mode

The Running Command mode can only be accessed via using the `:run` command, and is used to display the output of a terminal command. Quit this mode by pressing `Enter`.

### General

You can switch mode using these keys :
 - `Esc` allows you to go into COMMAND mode
 - `i` allows you to go into INSERT mode
 - `Enter` allows you to return to COMMAND mode when in RUNNING\_COMMAND mode
