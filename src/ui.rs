use crate::{
    ansi::colors::{AnsiRGB, BLUE, BOLD, GREEN, MAGENTA, RED, RESET, YELLOW},
    ansi::{CLEAR, Csi, EL},
    cryptography::check_hash,
    storage::{PasswordArray, get_master_password},
};
use getch_rs::{Getch, Key};
use rand::{Rng, SeedableRng, rngs::StdRng};
use secrecy::{ExposeSecret, SecretString};
use std::{
    collections::HashSet,
    fmt::Display,
    fs,
    io::{self, Write},
    iter::zip,
    ops::{Add, Sub},
};

const V: &str = "✔";
const W: &str = "⚠︎";
const HELP_MESSAGE: &str = "There are a total of 7 commands (which have alaises):\n\nchoose (no other alias): Chooses a directory. Only accepts directories with the correct files\ncd (no other alias): Changes current working directory\nls (no other alias): Lists the contents of the current working directory\nexit (q, quit, ex): Exits the program\nclear (c, cls): clears the screen\nnew (init, new_session, make): clears the screen and prompts the user for the new directories name and the master password to store the hash in the master_password file\nhelp (h, ?): Shows this help\n\nUsage:\n\nCommands with no arguments: ls, exit, clear, help, new\n\ncd: cd {dirname}\nchoose: choose {dirname}";

const YESES: [&str; 15] = [
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Token {
    Command(String),
    Text(String),
    Whitespace(char),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(i) => write!(f, "{i}"),
            Self::Whitespace(w) => write!(f, "{w}"),
            Self::Command(c) => write!(f, "{BLUE}{c}{RESET}"),
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
    pub fn new(options: Vec<String>, prompt: String, icon: String) -> Self {
        let selection = LimitedUint {
            value: 0,
            minimum: 0,
            maximum: options.len() - 1,
        };
        Self {
            options,
            selection,
            prompt,
            icon,
        }
    }

    pub fn interact(&mut self) -> usize {
        println!("{CLEAR}");
        let getch = Getch::new();
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
        password.chars().collect::<HashSet<char>>().iter().count(),
    );
    let special = length - (upper + digits + lower);
    let results: [bool; 5] = [length < 16, upper < 5, unique < 8, digits < 5, special < 4];
    let messages = [
        "The length of the password should be 16 characters long",
        "The password should have atleast 5 capital letters",
        "The password should have atleast 8 unique characters",
        "The password should have atleast 5 digits",
        "The password should have atleast 4 special characters",
    ];
    for (message, suggestion) in zip(messages, results) {
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
    let mut buf = std::io::stdout();
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
        let p = p.unwrap();
        if p.path().is_dir() {
            println!(
                "{BLUE}{BOLD}{}{RESET}",
                p.path().as_path().file_name().unwrap().to_str().unwrap()
            );
            continue;
        }
        println!(
            "{}",
            p.path().as_path().file_name().unwrap().to_str().unwrap()
        );
    }
}

fn parse(string: &String, commands: &[String]) -> Vec<Token> {
    let mut buffer: String = String::new();
    let mut tokens: Vec<Token> = vec![];
    let mut is_first_command: bool = true;
    let mut no_text_token: bool = true;
    for character in string.chars() {
        if [' ', '\t'].contains(&character) {
            if commands.contains(&buffer) && is_first_command && no_text_token {
                tokens.push(Token::Command(buffer.clone()));
                buffer.clear();
                is_first_command = false;
            } else if !buffer.is_empty() {
                tokens.push(Token::Text(buffer.clone()));
                buffer.clear();
                no_text_token = false
            }
            tokens.push(Token::Whitespace(character));
        } else {
            buffer.push(character)
        }
    }
    if !buffer.is_empty() {
        if commands.contains(&buffer) && is_first_command && no_text_token {
            tokens.push(Token::Command(buffer.clone()));
            buffer.clear();
        } else {
            tokens.push(Token::Text(buffer.clone()));
            buffer.clear();
        }
    }
    tokens
}

fn highlight(commands: &[String], string: &String) -> String {
    let mut result = String::new();
    for token in parse(string, commands) {
        result.push_str(format!("{token}").as_str())
    }
    result
}

pub fn inputi(
    prompt: impl Display,
    default: String,
    highlight_text: bool,
    commands: &[String],
) -> String {
    let getch = Getch::new();
    let mut buffer = String::new();
    let mut stdout = io::stdout();
    print!("{prompt}");
    let _ = stdout.flush();
    loop {
        let chr = getch.getch();
        match chr {
            Ok(Key::Char('\r')) => {
                if buffer.is_empty() && !default.is_empty() {
                    return default;
                } else if buffer.is_empty() {
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
        if prompt.to_string().lines().count() == 2 {
            print!(
                "{}{}{}{}{}",
                Csi::CPL.ansi(),
                Csi::CPL.ansi(),
                Csi::El(EL::EL2).ansi(),
                Csi::CNL.ansi(),
                Csi::El(EL::EL2).ansi(),
            );
            let _ = stdout.flush();
        } else {
            print!(
                "{}{}{}",
                Csi::CPL.ansi(),
                Csi::CNL.ansi(),
                Csi::El(EL::EL2).ansi()
            );
            let _ = stdout.flush();
        }
        if highlight_text {
            print!("{prompt}{}", highlight(commands, &buffer));
            let _ = stdout.flush();
        } else {
            print!("{prompt}{buffer}");
            let _ = stdout.flush();
        }
        let _ = stdout.flush();
    }
}

fn process_alias(alias: &str) -> &str {
    let alias = alias.trim();
    if ["choose", "cd", "ls", "exit", "clear", "new", "help"].contains(&alias) {
        return alias;
    } else if ["q", "quit", "ex"].contains(&alias) {
        return "exit";
    } else if ["c", "cls"].contains(&alias) {
        return "clear";
    } else if ["init", "new_session", "make", "code"].contains(&alias) {
        return "new";
    } else if ["h", "?"].contains(&alias) {
        return "help";
    }
    ""
}

fn new_directory() -> (String, SecretString, bool) {
    loop {
        let directory_name: String = inputi("Directory name: ", String::new(), false, &[]);
        if !fs::exists(&directory_name).unwrap() {
            let master_password = new_password_input("Master password: ");
            crate::storage::initialize_directory(&directory_name, master_password.expose_secret());
            return (directory_name, master_password, true);
        }
        println!();
    }
}

fn input(prompt: &[u8]) -> String {
    let mut buffer = String::new();
    let mut stdout = std::io::stdout();
    let _ = stdout.write(prompt);
    stdout.flush().unwrap();
    std::io::stdin()
        .read_line(&mut buffer)
        .expect("Something went wrong");
    buffer
}

fn getcwd() -> String {
    std::env::current_dir()
        .unwrap()
        .as_path()
        .to_str()
        .unwrap()
        .to_string()
}

fn check_master_password(directory_name: &str, input: &str) -> bool {
    let hashed_master_password = get_master_password(directory_name).unwrap();
    if !check_hash(input, &hashed_master_password[0]) {
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
    if std::env::set_current_dir(format!("{}/{}", getcwd(), directory_name).trim()).is_err() {
        eprintln!("Something went wrong")
    }
}

/// Gives a prompt to the user to choose a directory
pub fn directory_selector() -> (String, SecretString, bool) {
    loop {
        let current_directory = getcwd();
        let usr = inputi(
            format!("{BLUE}{current_directory}{RESET} {MAGENTA}❯ {RESET}"),
            String::new(),
            true,
            &[
                "exit".to_string(),
                "ex".to_string(),
                "q".to_string(),
                "quit".to_string(),
                "new".to_string(),
                "init".to_string(),
                "make".to_string(),
                "help".to_string(),
                "new_session".to_string(),
                "ls".to_string(),
                "choose".to_string(),
                "cd".to_string(),
                "clear".to_string(),
                "help".to_string(),
                "h".to_string(),
                "c".to_string(),
                "cls".to_string(),
                "?".to_string(),
            ],
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
        let (command, command_input): (&str, &str) = (process_alias(sp[0]), sp[1]);
        if command == "cd" {
            cd(command_input);
        } else if command == "choose" {
            let directory_name: String =
                format!("{}/{}", getcwd(), command_input).trim().to_string();
            if !crate::storage::verify_directory(&directory_name) {
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
        let number = inputi(prompt, default.clone(), false, &[]);
        if !number.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let number: i32 = number.parse().unwrap();
        return number;
    }
}

/// a massive match statement used for the functionality of the program
/// this shouldn't be used any other projects
pub fn run(index: usize, password_array: &mut PasswordArray) {
    match index {
        0 => {
            let service = inputi("Service: ", String::new(), false, &[]);
            let password = new_password_input("Password: ");
            let result = password_array.add_password(service, password);
            if result.is_err() {
                println!("{}", result.unwrap_err());
                pause();
            }
        }
        1 => {
            let service = inputi("Service: ", String::new(), false, &[]);
            let new_password = new_password_input("Password: ");
            let result = password_array.edit_password(service, new_password);
            if result.is_err() {
                println!("{}", result.unwrap_err());
                pause();
            }
        }
        2 => {
            let service = inputi("Service: ", String::new(), false, &[]);
            let result = password_array.remove_password(service);
            if result.is_err() {
                println!("{}", result.unwrap_err());
                pause();
            }
        }
        3 => {
            println!("{}", password_array.table());
            pause();
        }
        4 => {
            let generated_password = generate_password(
                prompt_number("Length of generated password: ", "14".to_string())
                    .try_into()
                    .unwrap(),
            );
            println!("\nGenerated password: {generated_password}");
            let answer = input(b"Do you want to add this password? ");
            if YESES.iter().any(|y| *y == answer.to_lowercase().trim()) {
                let service = inputi("Service: ", String::new(), false, &[]);
                let res = password_array
                    .add_password(service, secrecy::SecretString::from(generated_password));
                if res.is_err() {
                    println!("{}", res.unwrap_err())
                }
            }
        }
        5 => {
            password_array.save(true);
            std::process::exit(0)
        }
        _ => {}
    }
}
