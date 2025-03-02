//! The documentation in this program is not really good
//! so don't set your expectations too high

mod ansi;
mod cryptography;
mod storage;
mod ui;

use storage::PasswordArray;
use ui::{Menu, directory_selector, run};

fn main() {
    let mut menu = Menu::new(
        vec![
            "1. Add a password".to_string(),
            "2. Edit a password".to_string(),
            "3. Remove a password".to_string(),
            "4. List passwords".to_string(),
            "5. Generate password".to_string(),
            "6. Save & quit".to_string(),
        ],
        "nspm v1.0.0".to_string(),
        ">".to_string(),
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
