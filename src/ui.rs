use crate::{
    ansi::{CLEAR, Csi, EL, colors::AnsiRGB, constants::*},
    cryptography::check_hash,
    storage::{get_master_password, initialize_directory, verify_directory},
};
use getch_rs::{Getch, Key};
use rand::{Rng, SeedableRng, rngs::StdRng};
use secrecy::{ExposeSecret, SecretString};
use std::{
    collections::HashSet,
    fmt::Display,
    fs,
    io::{self, Write},
    mem::take,
};
use std::{path::Path, process::exit};

const V: &str = "✔";
const W: &str = "⚠︎";
const HELP_MESSAGE: &str = "There are a total of 7 commands (which have alaises):\n\nchoose (no other alias): Chooses a directory. Only accepts directories with the correct files\ncd (no other alias): Changes current working directory\nls (no other alias): Lists the contents of the current working directory\nexit (q, quit, ex): Exits the program\nclear (c, cls): clears the screen\nnew (init, new_session, make): clears the screen and prompts the user for the new directories name and the master password to store the hash in the master_password file\nhelp (h, ?): Shows this help\n\nUsage:\n\nCommands with no arguments: ls, exit, clear, help, new\n\ncd: cd {dirname}\nchoose: choose {dirname}";

pub const YESES: [&str; 15] = [
    "y",
    "yes",
    "ye",
    "yahoo",
    "yes i want to save this password to be able to access it again later",
    "save",
    "ok",
    "k",
    "sure",
    "fine",
    "finally",
    "just save",
    "es",
    "s",
    "se",
];

const CHARS: [&str; 94] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L",
    "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "0", "1", "2", "3", "4",
    "5", "6", "7", "8", "9", "!", "\"", "#", "$", "%", "&", "'", "(", ")", "*", "+", ",", "-", ".",
    "/", ":", ";", "<", "=", ">", "?", "@", "[", "\\", "]", "^", "_", "`", "{", "|", "}", "~",
];

pub const ALL_FLAGS: [InputFlags; 4] = [
    InputFlags::HighlightInput,
    InputFlags::IsBlacklist,
    InputFlags::DenyEmptyInput,
    InputFlags::AllowBlacklist,
];

pub const NO_FLAGS: &[InputFlags; 0] = &[];
pub const NO_COMMANDS: &[String; 0] = &[];

const COMMANDS: [&str; 7] = ["choose", "cd", "ls", "exit", "clear", "new", "help"];

const EXIT_ALIASES: [&str; 3] = ["q", "quit", "ex"];
const NEW_ALIASES: [&str; 3] = ["new", "new_session", "make"];
const CLEAR_ALIASES: [&str; 2] = ["c", "cls"];
const HELP_ALIASES: [&str; 2] = ["h", "?"];

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Token {
    InvalidCommand(String),
    Command(String),
    Text(String),
    Whitespace(char),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(t) => write!(f, "{t}"),
            Self::Whitespace(w) => write!(f, "{w}"),
            Self::Command(c) => write!(f, "{BLUE}{c}{RESET}"),
            Self::InvalidCommand(ic) => write!(f, "{RED}{ic}{RESET}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputFlags {
    DenyEmptyInput,
    HighlightInput,
    IsBlacklist,
    AllowBlacklist,
}

#[derive(Debug)]
pub struct ProgressBar<'a> {
    n: u32,
    d: u32,
    left: char,
    right: char,
    filled: &'a str,
    unfilled: &'a str,
    stop1: AnsiRGB,
    stop2: AnsiRGB,
    stop3: AnsiRGB,
    length: usize,
}

impl Display for ProgressBar<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let percent: f64 = self.n as f64 / (self.d as f64 / self.length as f64);
        let usp: usize = percent as usize;
        let (left, right, gradient_color, filled, unfilled) = (
            self.left,
            self.right,
            self.stop1.gradient(
                (self.n as f64 * 100.0) / self.d as f64,
                self.stop2,
                self.stop3,
            ),
            self.filled.repeat(usp),
            self.unfilled.repeat(self.length - usp),
        );
        write!(f, "{left}{gradient_color}{filled}{RESET}{unfilled}{right}",)
    }
}

impl ProgressBar<'_> {
    /// Creates a new progress bar
    pub fn new(d: u32) -> Self {
        Self {
            n: 0,
            d,
            left: '[',
            right: ']',
            filled: "#",
            unfilled: "─",
            length: 50,
            stop1: AnsiRGB { r: 255, g: 0, b: 0 },
            stop2: AnsiRGB {
                r: 255,
                g: 255,
                b: 0,
            },
            stop3: AnsiRGB {
                r: 50,
                g: 255,
                b: 0,
            },
        }
    }
    pub fn increase_n(&mut self) {
        self.n += 1;
    }
}

pub struct MenuConfig {
    pub prompt: String,
    pub icon: String,
}

/// A way simpler [Select](<https://docs.rs/dialoguer/latest/dialoguer/struct.Select.html>)
pub struct Menu {
    selection: VecIndex<String>,
    prompt: String,
    icon: String,
}

impl Menu {
    pub fn new(menu_config: MenuConfig, options: Vec<String>) -> Self {
        let selection = VecIndex::new(options);
        Self {
            selection,
            prompt: menu_config.prompt,
            icon: menu_config.icon,
        }
    }

    pub fn interact(&mut self) -> usize {
        println!("{}", Csi::Hide);
        println!("{CLEAR}");
        let getch = Getch::new();
        println!("{}", self.prompt);
        self.print_items();
        loop {
            let chr = getch.getch();
            match chr {
                Ok(Key::Char('\r')) => {
                    println!("{}", Csi::Show);
                    println!("{CLEAR}");
                    return self.selection.index;
                }
                Ok(Key::Char('j')) | Ok(Key::Down) | Ok(Key::Char('l')) => {
                    self.selection.next();
                }
                Ok(Key::Up) | Ok(Key::Char('k')) | Ok(Key::Backspace) | Ok(Key::Delete)
                | Ok(Key::Char('h')) => {
                    self.selection.prev();
                }
                Ok(Key::Ctrl('c')) => exit(1),
                Ok(_key) => {}
                Err(e) => eprintln!("{e}"),
            }
            println!("{CLEAR}");
            println!("{}", self.prompt);
            self.print_items();
        }
    }
    fn print_items(&self) {
        let space = " ".repeat(self.icon.len() + 1);
        for index in 0..self.selection.vector.len() {
            if index == self.selection.index {
                println!(
                    "{} {}",
                    self.icon,
                    self.selection.vector.get(index).unwrap()
                );
                continue;
            }
            println!("{}{}", space, self.selection.vector.get(index).unwrap())
        }
    }
}

struct VecIndex<T> {
    vector: Vec<T>,
    index: usize,
}

impl<T> VecIndex<T> {
    fn new(vector: Vec<T>) -> Self {
        Self { vector, index: 0 }
    }
    fn next(&mut self) {
        if self.index + 1 >= self.vector.len() {
            self.index = 0;
        } else {
            self.index += 1
        }
    }
    fn prev(&mut self) {
        if self.index == 0 {
            self.index = self.vector.len() - 1
        } else {
            self.index -= 1
        }
    }
}

// this is ugly
fn all_commands() -> Vec<String> {
    let mut commands: Vec<String> = COMMANDS.iter().map(|s| s.to_string()).collect();
    commands.append(&mut EXIT_ALIASES.iter().map(|s| s.to_string()).collect());
    commands.append(
        &mut CLEAR_ALIASES
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
    );
    commands.append(
        &mut NEW_ALIASES
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
    );
    commands.append(
        &mut HELP_ALIASES
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
    );
    commands
}

fn process_alias(alias: &str) -> &str {
    let alias = alias.trim();
    if COMMANDS.contains(&alias) {
        return alias;
    } else if EXIT_ALIASES.contains(&alias) {
        return "exit";
    } else if CLEAR_ALIASES.contains(&alias) {
        return "clear";
    } else if NEW_ALIASES.contains(&alias) {
        return "new";
    } else if HELP_ALIASES.contains(&alias) {
        return "help";
    }
    ""
}

fn directory_selector_prompt(format_string: &str) -> String {
    let mut on_percent = false;
    let mut res = String::new();
    for character in format_string.chars().chain("%R".chars()) {
        // BOLD: %B
        // RESET: %R
        // BLACK: %l
        // RED: %r
        // GREEN: %g
        // YELLOW: %y
        // BLUE: %b
        // MAGENTA: %m
        // CYAN: %c
        // WHITE: %w
        // short_path: %s
        // absolute_path: %S
        if on_percent {
            on_percent = false;
            match character {
                'R' => res.push_str(RESET),
                'l' => res.push_str(BLACK),
                'B' => res.push_str(BOLD),
                'r' => res.push_str(RED),
                'g' => res.push_str(GREEN),
                'y' => res.push_str(YELLOW),
                'b' => res.push_str(BLUE),
                'm' => res.push_str(MAGENTA),
                'c' => res.push_str(CYAN),
                'w' => res.push_str(WHITE),
                's' => res.push_str(&getcwd_short()),
                'S' => res.push_str(&getcwd()),
                '%' => res.push('%'),
                c => res.push(c),
            }
        } else if character == '%' {
            on_percent = true;
        } else {
            res.push(character);
        }
    }
    res
}

fn evaluate_password(password: &str) {
    let (length, upper, digits, lower, unique) = (
        password.chars().count(),
        password.chars().filter(|s| s.is_uppercase()).count(),
        password.chars().filter(|s| s.is_numeric()).count(),
        password.chars().filter(|s| s.is_lowercase()).count(),
        password.chars().collect::<HashSet<char>>().len(),
    );
    let special = length - (upper + digits + lower);
    let results: [bool; 5] = [length < 15, upper < 4, unique < 7, digits < 4, special < 4];
    let messages = [
        "The length of the password should be 15 characters long",
        "The password should have atleast 4 capital letters",
        "The password should have atleast 7 unique characters",
        "The password should have atleast 4 digits",
        "The password should have atleast 4 special characters",
    ];
    for (message, suggestion) in messages.iter().zip(results) {
        if suggestion {
            println!("{W} {YELLOW}{message}{RESET}");
            continue;
        }
        println!("{V}{GREEN} {message}{RESET}");
    }
}

/// Makes a password prompt with password suggestions
pub fn new_password_input(prompt: impl Display) -> secrecy::SecretString {
    let getch = Getch::new();
    let mut password = String::new();
    println!("{CLEAR}");
    println!("{prompt}");
    evaluate_password(&password);
    loop {
        let chr = getch.getch();
        match chr {
            Ok(Key::Char('\r')) => return secrecy::SecretString::from(password),
            Ok(Key::Char(c)) => {
                password.push(c);
            }
            Ok(Key::Backspace) => {
                password.pop();
            }
            Ok(Key::Delete) => {
                password.pop();
            }
            Ok(Key::Ctrl('c')) => exit(1),
            Ok(_key) => {}
            Err(e) => eprintln!("{e}"),
        }
        println!("{CLEAR}");
        println!("{prompt}{}", "*".repeat(password.len()));
        evaluate_password(&password);
    }
}

/// Makes a password prompt with no password suggestions
pub fn password_input(prompt: impl Display) -> SecretString {
    let getch = Getch::new();
    let mut password = String::new();
    println!("{CLEAR}");
    print!("{prompt}");
    let mut buf = io::stdout();
    let _ = buf.flush();
    loop {
        let chr = getch.getch();
        match chr {
            Ok(Key::Char('\r')) => return SecretString::from(password),
            Ok(Key::Char(c)) => {
                password.push(c);
            }
            Ok(Key::Backspace) => {
                password.pop();
            }
            Ok(Key::Delete) => {
                password.pop();
            }
            Ok(Key::Ctrl('c')) => exit(1),
            Ok(_key) => {}
            Err(e) => eprintln!("{e}"),
        }
        println!("{CLEAR}");
        print!("{prompt}{}", "*".repeat(password.len()));
        let _ = buf.flush();
    }
}

fn list_directory(path: &str) {
    for p in fs::read_dir(path).unwrap() {
        let path = p.unwrap().path();
        let path_str = path.file_name().unwrap().to_str().unwrap();
        if path.is_dir() {
            println!("{BLUE}{BOLD}{path_str}{RESET}");
            continue;
        }
        println!("{path_str}");
    }
}

fn parse(string: &String, commands: &[String], parse_invalid: bool) -> Vec<Token> {
    if string.is_empty() {
        return Vec::new();
    }
    let mut buffer: String = String::new();
    let mut tokens: Vec<Token> = vec![];
    let mut is_first_command: bool = true;
    let mut no_text_token: bool = true;
    for character in string.chars() {
        if [' ', '\t'].contains(&character) {
            if buffer.is_empty() {
                tokens.push(Token::Whitespace(character));
                continue;
            } else if commands.contains(&buffer) && is_first_command && no_text_token {
                let t = if parse_invalid {
                    Token::InvalidCommand(take(&mut buffer))
                } else {
                    Token::Command(take(&mut buffer))
                };
                tokens.push(t);
                is_first_command = false;
            } else {
                tokens.push(Token::Text(take(&mut buffer)));
                no_text_token = false;
            }
            tokens.push(Token::Whitespace(character));
        } else {
            buffer.push(character)
        }
    }
    if !buffer.is_empty() {
        if commands.contains(&buffer) && is_first_command && no_text_token {
            let t = if parse_invalid {
                Token::InvalidCommand(take(&mut buffer))
            } else {
                Token::Command(take(&mut buffer))
            };
            tokens.push(t);
        } else {
            tokens.push(Token::Text(take(&mut buffer)));
        }
    }
    tokens
}

pub fn input(
    prompt: impl Display,
    default: String,
    commands: &[String],
    flags: &[InputFlags],
) -> String {
    let (deny_empty_input, is_blacklist, highlight_text, deny_blacklist) = (
        flags.contains(&InputFlags::DenyEmptyInput),
        flags.contains(&InputFlags::IsBlacklist),
        flags.contains(&InputFlags::HighlightInput),
        !flags.contains(&InputFlags::AllowBlacklist),
    );
    let getch = Getch::new();
    let mut buffer = String::new();
    let mut stdout = io::stdout();
    print!("{prompt}");
    let _ = stdout.flush();
    loop {
        let chr = getch.getch();
        match chr {
            Ok(Key::Char('\r')) => {
                if deny_empty_input && buffer.is_empty() {
                    continue;
                }
                if buffer.is_empty() && !default.is_empty() {
                    return default;
                }
                if is_blacklist
                    && deny_blacklist
                    && parse(&buffer, commands, true)
                        .iter()
                        .any(|token| matches!(token, &Token::InvalidCommand(_)))
                {
                    continue;
                }
                println!();
                return buffer;
            }
            Ok(Key::Char(c)) => {
                buffer.push(c);
            }
            Ok(Key::Backspace) | Ok(Key::Delete) => {
                buffer.pop();
            }
            Ok(Key::Ctrl('c')) | Ok(Key::Ctrl('z')) => exit(1),
            Ok(_key) => {}
            Err(e) => eprintln!("{e}"),
        }
        print!("{}{}{}", Csi::CPL, Csi::CNL, Csi::El(EL::EL2));
        let _ = stdout.flush();
        if highlight_text {
            print!(
                "{prompt}{}",
                parse(&buffer, commands, is_blacklist)
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .concat()
            );
            let _ = stdout.flush();
        } else {
            print!("{prompt}{buffer}");
            let _ = stdout.flush();
        }
        let _ = stdout.flush();
    }
}
fn new_directory() -> Result<(String, SecretString, bool), String> {
    loop {
        let directory_name: String = input(
            "Directory name: ",
            String::new(),
            &fs::read_dir(getcwd())
                .unwrap()
                .map(|p| p.unwrap().file_name().into_string().unwrap())
                .collect::<Vec<String>>(),
            &[
                InputFlags::IsBlacklist,
                InputFlags::DenyEmptyInput,
                InputFlags::HighlightInput,
            ],
        );
        let master_password = new_password_input("Master password: ");
        initialize_directory(&directory_name, master_password.expose_secret())?;
        println!();
        return Ok((directory_name, master_password, true));
    }
}

/// Gets current working directory
///
/// Panics if current working directory has invalid UTF-8
pub fn getcwd() -> String {
    std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

/// Shortens home as ~ in the current working directory path
pub fn getcwd_short() -> String {
    let current_directory: String = getcwd();
    let home_dir = std::env::vars().find(|key_value| key_value.0 == "HOME".to_string());
    if home_dir.is_none() {
        return current_directory;
    }
    let home_dir = home_dir.unwrap().1;
    if current_directory.len() < home_dir.len() {
        return current_directory;
    }
    if current_directory[..home_dir.len()] != home_dir {
        return current_directory;
    }
    current_directory.replacen(&home_dir, "~", 1)
}

fn check_master_password(directory_name: &str, input: &str) -> bool {
    let hashed_master_password = get_master_password(directory_name).unwrap();
    if !check_hash(input, &hashed_master_password) {
        return false;
    }
    true
}

pub fn prompt_master_password(directory_name: &str) -> SecretString {
    for _ in 1..=3 {
        let master = password_input("Master password: ");
        println!();
        if !check_master_password(directory_name, master.expose_secret()) {
            continue;
        }
        return master;
    }
    eprintln!("3 incorrect password attempts");
    exit(1)
}

fn process_command(command: &str) {
    if command == "ls" {
        list_directory(&getcwd());
    } else if command == "exit" {
        exit(0)
    } else if command == "clear" {
        println!("{CLEAR}");
    } else if command == "help" {
        println!("{HELP_MESSAGE}");
    } else {
        println!("{RED}{BOLD}Command not found{RESET}");
    }
}

/// Gives a prompt to the user to choose a directory
pub fn directory_selector(format_string: String) -> Result<(String, SecretString, bool), String> {
    let commands = all_commands();
    let mut prompt = directory_selector_prompt(&format_string);
    loop {
        let usr = input(
            prompt.clone(),
            String::new(),
            &commands,
            &[InputFlags::HighlightInput],
        );
        let sp: Vec<&str> = usr.split_whitespace().collect();
        if sp.is_empty() {
            continue;
        } else if sp.len() == 1 {
            let command = process_alias(sp[0]);
            if ["cd", "choose"].contains(&command) {
                eprintln!("{RED}{BOLD}This command needs 2 arguments{RESET}");
                continue;
            }
            if command == "new" {
                return Ok(new_directory()?);
            }
            process_command(command);
            continue;
        }
        let (command, command_input): (&str, &str) = (process_alias(sp[0]), &sp[1..].join(" "));
        if command == "cd" {
            let cd_result = std::env::set_current_dir(command_input);
            if cd_result.is_err() {
                eprintln!("{}", cd_result.unwrap_err());
                continue;
            }
            prompt = directory_selector_prompt(&format_string);
        } else if command == "choose" {
            let directory_name: String = Path::new(&getcwd())
                .join(command_input)
                .to_str()
                .unwrap()
                .to_string();
            if !verify_directory(&directory_name) {
                println!(
                    "Either the directory provided doesn't exist or it doesn't have the correct structure"
                );
                continue;
            }
            let master_password = prompt_master_password(&directory_name);
            return Ok((directory_name, master_password, false));
        } else {
            println!("{RED}{BOLD}Command not found{RESET}");
        }
    }
}

/// "Press any key to continue..." recreation
pub fn pause() {
    let getch = Getch::new();
    let mut buf = io::stdout();
    print!("Press any key to continue...");
    let _ = buf.flush();
    let chr = getch.getch();
    match chr {
        Ok(Key::Ctrl('c')) => exit(1),
        Ok(_key) => {}
        Err(e) => eprintln!("{e}"),
    }
}

/// Generates a random (hopefully) password
pub fn generate_password(length: u32) -> String {
    let mut os = StdRng::from_os_rng();
    let mut generated_password = String::new();
    for _ in 0..length {
        generated_password.push_str(CHARS.get(os.random_range(..CHARS.len())).unwrap());
    }
    generated_password
}

/// Prompts the user for a number
pub fn prompt_number(prompt: &str, default: String) -> u32 {
    loop {
        let number = input(prompt, default.clone(), NO_COMMANDS, NO_FLAGS);
        if number.bytes().any(|b| !b.is_ascii_digit()) {
            continue;
        }
        match number.parse::<u32>() {
            Ok(n) => return n,
            Err(err) => eprintln!("Error when parsing integer: {err}"),
        }
    }
}
