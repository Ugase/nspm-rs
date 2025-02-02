use crate::storage::PasswordArray;
use crate::{cryptography::check_hash, storage::get_master_password};
use dialoguer::{Input, Password, Select};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fs;
use std::io::{Read, Write, stdin, stdout};

const CLEAR: &str = "\x1b[H\x1b[2J\x1b[3J";

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

pub fn menu(items: &Vec<&str>) -> usize {
    println!("{}", CLEAR);
    let selection = Select::new()
        .with_prompt("nspm v0.2.0")
        .items(items)
        .interact()
        .unwrap();
    selection
}

pub fn password_input() -> String {
    let pass = Password::new().with_prompt("Password").interact().unwrap();
    pass
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

fn process_command(alias: &str) -> &str {
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

fn new_directory() -> [String; 3] {
    let directory_name: String = Input::new()
        .with_prompt("Directory name")
        .report(false)
        .interact_text()
        .expect("uhhhh");
    let master_password = Password::new()
        .with_prompt("Master password")
        .report(false)
        .interact()
        .expect("uhhhh");
    crate::storage::initialize_directory(&directory_name, &master_password);
    [directory_name, master_password, "true".to_string()]
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

pub fn directory_selector() -> [String; 3] {
    loop {
        let usr = input(format!("\x1b[94m{}\x1b[0m\n\x1b[95m❯ \x1b[0m", getcwd()).as_bytes());
        let sp: Vec<&str> = usr.split_whitespace().collect();
        if sp.is_empty() {
            continue;
        } else if sp.len() == 1 {
            let command = process_command(sp[0]);
            if ["cd", "choose"].contains(&command) {
                continue;
            }
            if command == "ls" {
                list_directory(&getcwd());
            } else if command == "exit" {
                std::process::exit(0)
            } else if command == "clear" {
                println!("{}", CLEAR);
            } else if command == "help" {
                println!("{}", HELP_MESSAGE);
            } else if command == "new" {
                return new_directory();
            };
        } else if sp.len() >= 2 {
            let (command, command_input): (&str, &str) = (process_command(sp[0]), sp[1]);
            if command == "cd" {
                if std::env::set_current_dir(format!("{}/{}", getcwd(), command_input).trim())
                    .is_err()
                {
                    println!("Something went wrong")
                }
            } else if command == "choose" {
                let directory_name: String =
                    format!("{}/{}", getcwd(), command_input).trim().to_string();
                if !crate::storage::verify_directory(&directory_name) {
                    println!(
                        "Either the directory provided doesn't exist or it doesn't have the correct files and folders"
                    );
                    continue;
                }
                let master_password = Password::new()
                    .with_prompt("Master password")
                    .report(false)
                    .validate_with(|input: &String| -> Result<(), &str> {
                        let hashed_master_password = get_master_password(&directory_name).unwrap();
                        if check_hash(
                            input.as_str(),
                            &hashed_master_password[1],
                            &hashed_master_password[0],
                        ) {
                            Ok(())
                        } else {
                            Err("Incorrect master password")
                        }
                    })
                    .interact()
                    .expect("uhhhh");
                return [directory_name, master_password, "false".to_string()];
            }
        }
    }
}

// Code from u/K900_ on reddit
pub fn pause() {
    let mut stdout = stdout();
    stdout.write_all(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read_exact(&mut [0]).unwrap();
}

pub fn action(index: u8, password_array: &mut PasswordArray, directory_name: &str) {
    match index {
        0 => {
            let service = Input::new().with_prompt("Service").interact().unwrap();
            let password = password_input();
            let _ = password_array.add_password(service, password);
        }
        1 => {
            let service = Input::new().with_prompt("Service").interact().unwrap();
            let new_password = password_input();
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
            let chars = [
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
                "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "A", "B", "C", "D", "E", "F",
                "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V",
                "W", "X", "Y", "Z", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "!", "\"",
                "#", "$", "%", "&", "'", "(", ")", "*", "+", ",", "-", ".", "/", ":", ";", "<",
                "=", ">", "?", "@", "[", "\\", "]", "^", "_", "`", "{", "|", "}", "~",
            ];
            let mut os = StdRng::from_os_rng();
            let length = Input::new()
                .with_prompt("Length of generated password")
                .validate_with(|input: &String| -> Result<(), &str> {
                    if !input.bytes().all(|b| b.is_ascii_digit()) {
                        return Err("not a number");
                    };
                    Ok(())
                })
                .default("14".to_owned())
                .interact()
                .unwrap();
            let length: u32 = length.parse().unwrap();
            let mut generated_password = String::new();
            for _ in 0..length {
                generated_password.push_str(chars.get(os.random_range(..94usize)).unwrap());
            }
            println!("Generated password: {generated_password}");
            let answer = input(b"Do you want to add this password? ");
            if [
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
            ]
            .iter()
            .any(|y| *y == answer.to_lowercase().trim())
            {
                let service = Input::new().with_prompt("Service").interact().unwrap();
                let _ = password_array.add_password(service, generated_password);
            }
        }
        5 => {
            password_array.clone().save(directory_name);
            std::process::exit(0)
        }
        _ => {}
    }
}
