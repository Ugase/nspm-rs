use crate::{
    ansi::colors::{AnsiRGB, BLUE, BOLD, GREEN, RED, RESET, YELLOW},
    ansi::{CLEAR, Csi, EL},
    con::*,
    cryptography::check_hash,
    storage::{get_master_password, verify_directory},
};
use getch_rs::{Getch, Key};
use rand::{Rng, SeedableRng, rngs::StdRng};
use secrecy::{ExposeSecret, SecretString};
use std::path::Path;
use std::{
    collections::HashSet,
    fmt::Display,
    fs,
    io::{self, Write},
    ops::{Add, Sub},
};

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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Token {
    InvalidCommand(String),
    Command(String),
    Text(String),
    Whitespace(char),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputFlags {
    DenyEmptyInput,
    HighlightInput,
    IsBlacklist,
    AllowBlacklist,
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

pub struct Menu {
    options: Vec<String>,
    selection: LimitedUint,
    prompt: String,
    icon: String,
}

impl Menu {
    pub fn new(menu_config: MenuConfig, options: Vec<String>) -> Self {
        let selection = LimitedUint {
            value: 0,
            minimum: 0,
            maximum: options.len() - 1,
        };
        Self {
            options,
            selection,
            prompt: menu_config.prompt,
            icon: menu_config.icon,
        }
    }

    pub fn interact(&mut self) -> usize {
        println!("{CLEAR}");
        let getch = Getch::new();
        println!("{}", self.prompt);
        self.print_items();
        loop {
            let chr = getch.getch();
            match chr {
                Ok(Key::Char('\r')) => {
                    println!("{CLEAR}");
                    return self.selection.value;
                }
                Ok(Key::Char('j')) | Ok(Key::Down) | Ok(Key::Char('l')) => {
                    self.selection = self.selection
                        + LimitedUint {
                            value: 1,
                            minimum: 0,
                            maximum: 2,
                        }
                }
                Ok(Key::Up) | Ok(Key::Char('k')) | Ok(Key::Backspace) | Ok(Key::Delete)
                | Ok(Key::Char('h')) => {
                    self.selection = self.selection
                        - LimitedUint {
                            value: 1,
                            minimum: 0,
                            maximum: 2,
                        }
                }
                Ok(Key::Ctrl('c')) => std::process::exit(1),
                Ok(_key) => {}
                Err(e) => eprintln!("{e}"),
            }
            println!("{CLEAR}");
            println!("{}", self.prompt);
            self.print_items();
        }
    }
    fn print_items(&self) {
        let space = " ".repeat(self.icon.chars().count() + 1);
        for index in 0..self.options.len() {
            if index == self.selection.value {
                println!("{} {}", self.icon, *self.options.get(index).unwrap());
                continue;
            }
            println!("{}{}", space, *self.options.get(index).unwrap())
        }
    }
}

#[derive(Debug)]
pub struct ProgressBar<'a> {
    pub n: u32,
    pub d: u32,
    pub left: char,
    pub right: char,
    pub filled: &'a str,
    pub unfilled: &'a str,
    pub stop1: AnsiRGB,
    pub stop2: AnsiRGB,
    pub stop3: AnsiRGB,
    pub length: u32,
}

impl ProgressBar<'_> {
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
    pub fn increse_n(&mut self) {
        self.n += 1;
    }
}

impl Display for ProgressBar<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let percent: f64 = self.n as f64 / (self.d as f64 / self.length as f64);
        let usp: usize = percent as usize;
        let uthr: u32 = percent as u32;
        write!(
            f,
            "{0}{2}{3}{RESET}{4}{1}",
            self.left,
            self.right,
            self.stop1.gradient(
                (self.n as f64 * 100.0) / self.d as f64,
                self.stop2,
                self.stop3
            ),
            self.filled.repeat(usp),
            self.unfilled
                .repeat((self.length - uthr).try_into().unwrap()),
        )
    }
}

#[derive(Debug, Copy, Clone)]
struct LimitedUint {
    value: usize,
    minimum: usize,
    maximum: usize,
}

impl Add for LimitedUint {
    type Output = LimitedUint;
    fn add(self, rhs: Self) -> Self::Output {
        let res = LimitedUint {
            value: self.value + rhs.value,
            minimum: self.minimum,
            maximum: self.maximum,
        };
        if res.value <= self.maximum {
            res
        } else {
            LimitedUint {
                value: self.minimum,
                minimum: self.minimum,
                maximum: self.maximum,
            }
        }
    }
}

impl Sub for LimitedUint {
    type Output = LimitedUint;
    fn sub(self, rhs: Self) -> Self::Output {
        if rhs.value > self.value {
            return LimitedUint {
                value: self.maximum,
                minimum: self.minimum,
                maximum: self.maximum,
            };
        }
        let res = LimitedUint {
            value: self.value - rhs.value,
            minimum: self.minimum,
            maximum: self.maximum,
        };
        if res.value >= self.minimum {
            res
        } else {
            LimitedUint {
                value: self.maximum,
                minimum: self.minimum,
                maximum: self.maximum,
            }
        }
    }
}

fn evaluate_password(password: &str) {
    let (length, upper, digits, lower, unique) = (
        password.chars().count(),
        password.chars().filter(|s| s.is_uppercase()).count(),
        password.chars().filter(|s| s.is_ascii_digit()).count(),
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
            Ok(Key::Ctrl('c')) => std::process::exit(1),
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
            Ok(Key::Ctrl('c')) => std::process::exit(1),
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
        let path_str = path.as_path().file_name().unwrap().to_str().unwrap();
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
            if !buffer.is_empty() {
                if commands.contains(&buffer) && is_first_command && no_text_token {
                    tokens.push(if parse_invalid {
                        Token::InvalidCommand(buffer.clone())
                    } else {
                        Token::Command(buffer.clone())
                    });
                    buffer.clear();
                    is_first_command = false;
                } else {
                    tokens.push(Token::Text(buffer.clone()));
                    buffer.clear();
                    no_text_token = false;
                }
            }
            tokens.push(Token::Whitespace(character));
        } else {
            buffer.push(character)
        }
    }
    if !buffer.is_empty() {
        if commands.contains(&buffer) && is_first_command && no_text_token {
            if parse_invalid {
                tokens.push(Token::InvalidCommand(buffer.clone()))
            } else {
                tokens.push(Token::Command(buffer.clone()));
            }
            buffer.clear();
        } else {
            tokens.push(Token::Text(buffer.clone()));
            buffer.clear();
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
    let (deny_empty_input, is_blacklist, highlight_text, allow_blacklist) = (
        flags.contains(&InputFlags::DenyEmptyInput),
        flags.contains(&InputFlags::IsBlacklist),
        flags.contains(&InputFlags::HighlightInput),
        flags.contains(&InputFlags::AllowBlacklist),
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
                    && !allow_blacklist
                    && !parse(&buffer, commands, true)
                        .iter()
                        .filter(|t| match t {
                            Token::InvalidCommand(_) => true,
                            _ => false,
                        })
                        .collect::<Vec<&Token>>()
                        .is_empty()
                {
                    continue;
                }
                println!();
                return buffer;
            }
            Ok(Key::Char(c)) => {
                buffer.push(c);
            }
            Ok(Key::Backspace) => {
                buffer.pop();
            }
            Ok(Key::Delete) => {
                buffer.pop();
            }
            Ok(Key::Ctrl('c')) | Ok(Key::Ctrl('z')) => std::process::exit(1),
            Ok(_key) => {}
            Err(e) => eprintln!("{e}"),
        }
        print!(
            "{}{}{}",
            Csi::CPL.ansi(),
            Csi::CNL.ansi(),
            Csi::El(EL::EL2).ansi()
        );
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

fn new_directory() -> (String, SecretString, bool) {
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
        crate::storage::initialize_directory(&directory_name, master_password.expose_secret());
        println!();
        return (directory_name, master_password, true);
    }
}

pub fn getcwd() -> String {
    std::env::current_dir()
        .unwrap()
        .as_path()
        .to_str()
        .unwrap()
        .to_string()
}

pub fn getcwd_short() -> String {
    let current_directory: String = getcwd();
    let home_dir = std::env::vars().find(|key_value| key_value.0 == "HOME".to_string());
    if home_dir.is_none() {
        return current_directory;
    }
    let home_dir = home_dir.unwrap().1;
    if current_directory.len() >= home_dir.len() {
        return current_directory;
    }
    if current_directory[..home_dir.len()] != home_dir {
        return current_directory;
    }
    return current_directory.replacen(&home_dir, "~", 1);
}

fn check_master_password(directory_name: &str, input: &str) -> bool {
    let hashed_master_password = get_master_password(directory_name).unwrap();
    if !check_hash(input, &hashed_master_password) {
        return false;
    }
    true
}

fn prompt_master_password(directory_name: &str) -> SecretString {
    for _ in 1..=3 {
        let master = password_input("Master password: ");
        println!();
        if !check_master_password(directory_name, master.expose_secret()) {
            continue;
        }
        return master;
    }
    eprintln!("3 incorrect password attempts");
    std::process::exit(1)
}

fn process_command(command: &str) {
    if command == "ls" {
        list_directory(&getcwd());
    } else if command == "exit" {
        std::process::exit(0)
    } else if command == "clear" {
        println!("{CLEAR}");
    } else if command == "help" {
        println!("{HELP_MESSAGE}");
    } else {
        println!("{RED}{BOLD}Command not found{RESET}");
    }
}

/// Changes current working directory
pub fn cd(directory_name: &str) {
    let is_absolute_path = directory_name[0..1] == *"/";
    let future_cwd: String;
    if is_absolute_path {
        future_cwd = directory_name.to_string()
    } else {
        future_cwd = format!("{}/{}", getcwd(), directory_name)
    }
    let res = std::env::set_current_dir(future_cwd);
    if res.is_err() {
        let res = res.unwrap_err();
        eprintln!("{res}")
    }
}

/// Gives a prompt to the user to choose a directory
pub fn directory_selector() -> (String, SecretString, bool) {
    let commands = all_commands();
    loop {
        let usr = input(
            directory_selector_prompt(),
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
                continue;
            }
            if command == "new" {
                return new_directory();
            }
            process_command(command);
            continue;
        }
        let (command, command_input): (&str, &str) = (process_alias(sp[0]), &sp[1..].join(" "));
        if command == "cd" {
            cd(command_input);
        } else if command == "choose" {
            let directory_name: String = Path::new(&getcwd())
                .join(command_input)
                .to_str()
                .unwrap()
                .to_string();
            if !verify_directory(&directory_name) {
                println!(
                    "Either the directory provided doesn't exist or it doesn't have the correct files and folders"
                );
                continue;
            }
            let master_password = prompt_master_password(&directory_name);
            return (directory_name, master_password, false);
        } else {
            println!("{RED}{BOLD}Command not found{RESET}");
        }
    }
}

pub fn pause() {
    let getch = Getch::new();
    let mut buf = io::stdout();
    print!("Press any key to continue...");
    let _ = buf.flush();
    let chr = getch.getch();
    match chr {
        Ok(Key::Ctrl('c')) => std::process::exit(1),
        Ok(_key) => {}
        Err(e) => eprintln!("{e}"),
    }
}

/// Generates a random (hopefully) password
pub fn generate_password(length: u32) -> String {
    let mut os = StdRng::from_os_rng();
    let mut generated_password = String::new();
    for _ in 0..length {
        generated_password.push_str(CHARS.get(os.random_range(..94usize)).unwrap());
    }
    generated_password
}

/// Prompts the user for a number
pub fn prompt_number(prompt: &str, default: String) -> i32 {
    loop {
        let number = input(prompt, default.clone(), NO_COMMANDS, NO_FLAGS);
        if !number.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let number: i32 = number.parse().unwrap();
        return number;
    }
}
