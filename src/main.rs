//! The documentation in this program is not really good
//! so don't set your expectations too high

mod ansi;
mod con;
mod cryptography;
mod storage;
mod ui;

use con::*;
use storage::PasswordArray;
use ui::*;

fn main() {
    let mut menu = Menu::new(
        MenuConfig::default(),
        vec![
            "1. Add a password".to_string(),
            "2. Edit a password".to_string(),
            "3. Remove a password".to_string(),
            "4. List passwords".to_string(),
            "5. Generate password".to_string(),
            "6. Save & quit".to_string(),
        ],
    );
    let (directory, master_password, is_new) = directory_selector();
    if is_new {
        let mut password_array = PasswordArray::new(master_password, directory);
        loop {
            run(menu.interact(), &mut password_array);
        }
    }
    let mut password_array = PasswordArray::new(master_password, String::new());
    password_array.load(&directory, true).unwrap();
    loop {
        run(menu.interact(), &mut password_array);
    }
}

fn run(index: usize, password_array: &mut PasswordArray) {
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
            }
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
            }
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
            }
        }
        3 => {
            let table = password_array.table();
            println!("{table}");
            drop(table);
            pause();
        }
        4 => {
            let generated_password = generate_password(
                prompt_number("Length of generated password: ", "14".to_string())
                    .try_into()
                    .unwrap(),
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
