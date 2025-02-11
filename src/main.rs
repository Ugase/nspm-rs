use nspm::{
    storage::PasswordArray,
    ui::{action, directory_selector, menu},
};

fn main() {
    let menu_options = vec![
        "1. Add a password",
        "2. Edit a password",
        "3. Remove a password",
        "4. List passwords",
        "5. Generate password",
        "6. Save & quit",
    ];
    let prompt = "nspm v0.4.0";
    let (directory, master_password, is_new) = directory_selector();
    if is_new {
        let mut password_array = PasswordArray::new(&master_password);
        loop {
            action(
                menu(&menu_options, prompt).try_into().unwrap(),
                &mut password_array,
                &directory,
            );
        }
    }
    let mut password_array = PasswordArray::new(&master_password);
    password_array.load(&master_password, &directory).unwrap();
    loop {
        action(
            menu(&menu_options, prompt).try_into().unwrap(),
            &mut password_array,
            &directory,
        );
    }
}

//fn main() {
//    println!("{}", nspm::ui::new_password_input())
//}
