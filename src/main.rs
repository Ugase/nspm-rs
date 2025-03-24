//! The documentation in this program is not really good
//! so don't set your expectations too high

pub mod ansi;
pub mod cryptography;
pub mod storage;
pub mod ui;

use ansi::constants::*;
use clap::Parser;
use secrecy::SecretString;
use std::process::exit;
use storage::{PasswordArray, verify_directory};
use ui::{
    ALL_FLAGS, InputFlags, Menu, MenuConfig, NO_COMMANDS, NO_FLAGS, YESES, directory_selector,
    generate_password, input, new_password_input, pause, prompt_master_password, prompt_number,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("\0\0\0\0"))]
    /// The directory nspm uses, if not specified it'll bring up the directory selector
    directory: String,

    #[arg(short, long, default_value_t = String::from("nspm v1.0.0"))]
    /// Menu prompt
    prompt: String,

    #[arg(short, long, default_value_t = String::from(">"))]
    /// character(s) pointing to which option is currently selected
    icon: String,

    #[arg(short, long, default_value_t = String::from("%b%s%R %m❯ %R"), verbatim_doc_comment)]
    /// format string for directory prompt
    ///
    /// format string:  
    ///
    /// RESET: %R  
    /// BOLD: %B  
    /// BLACK: %l  
    /// RED: %r  
    /// GREEN: %g  
    /// YELLOW: %y  
    /// BLUE: %b  
    /// MAGENTA: %m  
    /// CYAN: %c  
    /// WHITE: %w  
    /// short_path: %s  
    /// absolute_path: %S  
    format_string: String,
}

fn main() {
    let mut modified = false;
    let args = Args::parse();
    let mut menu = Menu::new(
        MenuConfig {
            prompt: args.prompt,
            icon: args.icon,
        },
        vec![
            "1. Add a password".to_string(),
            "2. Edit a password".to_string(),
            "3. Remove a password".to_string(),
            "4. List passwords".to_string(),
            "5. Generate password".to_string(),
            "6. Save & quit".to_string(),
            "7. Quit".to_string(),
        ],
    );
    let (directory, master_password, is_new) = {
        if &args.directory == "\0\0\0\0" {
            let result = directory_selector(args.format_string);
            if result.is_err() {
                eprintln!("Something went wrong: {}", result.unwrap_err());
                exit(1)
            }
            result.unwrap()
        } else if !verify_directory(&args.directory) {
            eprintln!(
                "{RED}Error: The directory provided either doesn't have the correct structure or it doesn't exist{RESET}"
            );
            exit(1);
        } else {
            (
                args.directory.clone(),
                prompt_master_password(&args.directory),
                false,
            )
        }
    };
    let mut password_array = PasswordArray::new(master_password, directory);
    if !is_new {
        let result = password_array.load(true);
        if result.is_err() {
            let error = result.unwrap_err();
            eprintln!("{error}")
        }
    }
    loop {
        run(menu.interact(), &mut password_array, &mut modified);
    }
}

fn run(index: usize, password_array: &mut PasswordArray, password_array_modified: &mut bool) {
    match index {
        0 => {
            let service = input(
                "Service: ",
                String::new(),
                &password_array.get_services(),
                &ALL_FLAGS,
            );
            let password = new_password_input("Password: ");
            let result = password_array.add_password(service, password);
            if result.is_err() {
                println!("{}", result.unwrap_err());
                pause();
                return;
            }
            *password_array_modified = true
        }
        1 => {
            let service = input(
                "Service: ",
                String::new(),
                &password_array.get_services(),
                &[InputFlags::HighlightInput, InputFlags::DenyEmptyInput],
            );
            let new_password = new_password_input("Password: ");
            let result = password_array.edit_password(service, new_password);
            if result.is_err() {
                println!("{}", result.unwrap_err());
                pause();
                return;
            }
            *password_array_modified = true
        }
        2 => {
            let service = input(
                "Service: ",
                String::new(),
                &password_array.get_services(),
                &[InputFlags::HighlightInput, InputFlags::DenyEmptyInput],
            );
            let result = password_array.remove_password(service);
            if result.is_err() {
                println!("{}", result.unwrap_err());
                pause();
                return;
            }
            *password_array_modified = true
        }
        3 => {
            let table = password_array.table();
            println!("{table}");
            drop(table);
            pause();
        }
        4 => {
            let generated_password = generate_password(
                prompt_number("Length of generated password: ", "14".to_string()).into(),
            );
            println!("\nGenerated password: {generated_password}");
            let answer = input(
                "Do you want to add this password? ",
                "yes".to_string(),
                NO_COMMANDS,
                NO_FLAGS,
            );
            if YESES.iter().any(|y| *y == answer.to_lowercase().trim()) {
                let service = input(
                    "Service: ",
                    String::new(),
                    &password_array.get_services(),
                    &ALL_FLAGS,
                );
                let res =
                    password_array.add_password(service, SecretString::from(generated_password));
                if res.is_err() {
                    println!("{}", res.unwrap_err());
                    pause();
                    return;
                }
                *password_array_modified = true
            }
        }
        5 => {
            let result = password_array.save(true);
            if result.is_err() {
                let error = result.unwrap_err();
                eprintln!("\n{error}");
                exit(1)
            }
            exit(0)
        }
        6 => {
            if *password_array_modified {
                let answer = input(
                    "You have some unsaved changes, are you sure? ",
                    "no".to_string(),
                    NO_COMMANDS,
                    NO_FLAGS,
                );
                if YESES.iter().any(|y| *y == answer.to_lowercase().trim()) {
                    exit(0)
                } else {
                    return;
                }
            }
        }
        _ => {}
    }
}
