use crate::{
    cryptography::check_hash,
    storage::{PasswordArray, get_master_password},
};
use dialoguer::{Input, Select};
use getch_rs::{Getch, Key};
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::{
    collections::HashSet,
    fmt::Display,
    fs,
    io::{Read, Write, stdin, stdout},
};

const CLEAR: &str = "\x1b[H\x1b[2J\x1b[3J";
const V: &str = "✔ ";
const W: &str = "⚠️ ";
const HELP_MESSAGE: &str = "There are a total of 7 commands (which have alaises):

choose (no other alias): Chooses a directory. Only accepts directories with the correct files
cd (no other alias): Changes current working directory
ls (no other alias): Lists the contents of the current working directory
exit (q, quit, ex): Exits the program
clear (c, cls): clears the screen
new (init, new_session, make): clears the screen and prompts the user for the new directories name and the master password to store the hash in the master_password file
help (h, ?): Shows this help

Usage:

Commands with no arguments: ls, exit, clear, help, new

cd: cd {dirname}
choose: choose {dirname}";

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

/// Make a menu using [Select]
pub fn menu(items: &Vec<&str>, prompt: &str) -> usize {
    println!("{}", CLEAR);
    let selection = Select::new()
        .with_prompt(prompt)
        .items(items)
        .interact()
        .unwrap();
    selection
}

fn wrap_warning(a: impl Display) {
    println!("{W} \x1b[1;33m{a}\x1b[0m");
}

fn wrap_good(a: impl Display) {
    println!("{V} \x1b[1;32m{a}\x1b[0m");
}

fn evaluate_password(password: &str) {
    let mut uni = HashSet::new();
    for chr in password.chars() {
        uni.insert(chr);
    }
    let (length, upper, unique, digits, lower) = (
        password.chars().count(),
        password.chars().filter(|s| s.is_uppercase()).count(),
        uni.len(),
        password.chars().filter(|s| s.is_ascii_digit()).count(),
        password.chars().filter(|s| s.is_lowercase()).count(),
    );
    let special = length - (upper + digits + lower);
    if length < 16 {
        wrap_warning("The length of the password should be 16 characters long");
    } else {
        wrap_good("The length of the password should be 16 characters long");
    }
    if upper < 5 {
        wrap_warning("The password should have atleast 5 capital letters");
    } else {
        wrap_good("The password should have atleast 5 capital letters");
    }
    if unique < 8 {
        wrap_warning("The password should have atleast 8 unigue characters");
    } else {
        wrap_good("The password should have atleast 8 unigue characters");
    }
    if digits < 5 {
        wrap_warning("The password should have atleast 5 digits");
    } else {
        wrap_good("The password should have atleast 5 digits");
    }
    if special < 4 {
        wrap_warning("The password should have atleast 4 special characters");
    } else {
        wrap_good("The password should have atleast 4 special characters");
    }
}

/// Makes a password prompt with password suggestions
pub fn new_password_input(prompt: impl Display) -> String {
    let getch = Getch::new();
    let mut password = "".to_string();
    println!("{}", CLEAR);
    println!("{prompt}");
    evaluate_password(&password);
    loop {
        let chr = getch.getch();
        match chr {
            Ok(Key::Char('\r')) => return password,
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
        println!("{}", CLEAR);
        println!("{prompt}{}", "*".repeat(password.len()));
        evaluate_password(&password);
    }
}

/// Makes a password prompt with no password suggestions
pub fn password_input(prompt: impl Display) -> String {
    let getch = Getch::new();
    let mut password = "".to_string();
    println!("{}", CLEAR);
    println!("{prompt}");
    loop {
        let chr = getch.getch();
        match chr {
            Ok(Key::Char('\r')) => return password,
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
        println!("{}", CLEAR);
        println!("{prompt}{}", "*".repeat(password.len()));
    }
}

fn list_directory(path: &str) {
    for p in fs::read_dir(path).unwrap() {
        let p = p.unwrap();
        if p.path().is_dir() {
            println!(
                "\x1b[94;1m{}\x1b[0m",
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

fn process_alias(alias: &str) -> &str {
    let alias = alias.trim();
    if ["choose", "cd", "ls", "exit", "clear", "new", "help"].contains(&alias) {
        return alias;
    } else if ["q", "quit", "ex"].contains(&alias) {
        return "exit";
    } else if ["c", "cls"].contains(&alias) {
        return "clear";
    } else if ["init", "new_session", "make"].contains(&alias) {
        return "new";
    } else if ["h", "?"].contains(&alias) {
        return "help";
    }
    ""
}

fn new_directory() -> (String, String, bool) {
    let directory_name: String = Input::new()
        .with_prompt("Directory name")
        .report(false)
        .interact_text()
        .expect("uhhhh");
    let master_password = new_password_input("Master password: ");
    crate::storage::initialize_directory(&directory_name, &master_password);
    (directory_name, master_password, true)
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

fn check_master_password(directory_name: &str, input: &String) -> bool {
    let hashed_master_password = get_master_password(directory_name).unwrap();
    if !check_hash(
        input.as_str(),
        &hashed_master_password[1],
        &hashed_master_password[0],
    ) {
        return false;
    }
    true
}

fn prompt_master_password(directory_name: &str) -> String {
    loop {
        let master = password_input("Master password: ");
        if !check_master_password(directory_name, &master) {
            eprintln!("Incorrect master password");
            continue;
        }
        return master;
    }
}

fn process_command(command: &str) {
    if command == "ls" {
        list_directory(&getcwd());
    } else if command == "exit" {
        std::process::exit(0)
    } else if command == "clear" {
        println!("{}", CLEAR);
    } else if command == "help" {
        println!("{}", HELP_MESSAGE);
    }
}

/// Changes current working directory
pub fn cd(directory_name: &str) {
    if std::env::set_current_dir(format!("{}/{}", getcwd(), directory_name).trim()).is_err() {
        eprintln!("Something went wrong")
    }
}

/// Gives a prompt to the user to choose a directory
pub fn directory_selector() -> (String, String, bool) {
    loop {
        let usr = input(format!("\x1b[94m{}\x1b[0m\n\x1b[95m❯ \x1b[0m", getcwd()).as_bytes());
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
        }
    }
}

/// Code stolen from u/K900_ on reddit  
/// Press Enter to continue...
pub fn pause() {
    let mut stdout = stdout();
    stdout.write_all(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read_exact(&mut [0]).unwrap();
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
    let number = Input::new()
        .with_prompt(prompt)
        .validate_with(|input: &String| -> Result<(), &str> {
            if !input.bytes().all(|b| b.is_ascii_digit()) {
                return Err("not a number");
            };
            Ok(())
        })
        .default(default)
        .interact()
        .unwrap();
    let number: i32 = number.parse().unwrap();
    number
}

/// a massive match statement used for the functionality of the program
/// this shouldn't be used any other projects
pub fn action(index: u8, password_array: &mut PasswordArray, directory_name: &str) {
    match index {
        0 => {
            let service = Input::new().with_prompt("Service").interact().unwrap();
            let password = new_password_input("Password: ");
            let _ = password_array.add_password(service, password);
        }
        1 => {
            let service = Input::new().with_prompt("Service").interact().unwrap();
            let new_password = new_password_input("Password: ");
            let _ = password_array.edit_password(service, new_password);
        }
        2 => {
            let service = Input::new()
                .with_prompt("Service to remove")
                .interact()
                .unwrap();
            let _ = password_array.remove_password(service);
        }
        3 => {
            println!("{}", password_array.table);
            pause();
        }
        4 => {
            let generated_password = generate_password(
                prompt_number("Length of generated password", "14".to_string())
                    .try_into()
                    .unwrap(),
            );
            println!("Generated password: {generated_password}");
            let answer = input(b"Do you want to add this password? ");
            if YESES.iter().any(|y| *y == answer.to_lowercase().trim()) {
                let service = Input::new().with_prompt("Service").interact().unwrap();
                let _ = password_array.add_password(service, generated_password);
            }
        }
        5 => {
            password_array.save(directory_name);
            std::process::exit(0)
        }
        _ => {}
    }
}
