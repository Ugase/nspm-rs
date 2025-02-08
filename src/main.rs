use nspm::{
    storage::PasswordArray,
    ui::{action, directory_selector, menu},
};

fn main() {
    let items = vec![
        "1. Add a password",
        "2. Edit a password",
        "3. Remove a password",
        "4. List passwords",
        "5. Generate password",
        "6. Save & quit",
    ];
    let prompt = "nspm v0.3.0";
    let directory_name = directory_selector();
    if directory_name[2].parse::<bool>().unwrap() {
        let mut password_array = PasswordArray::new(&directory_name[1]);
        loop {
            action(
                menu(&items, prompt).try_into().unwrap(),
                &mut password_array,
                &directory_name[0],
            );
        }
    }
    let mut password_array = PasswordArray::new(&directory_name[1]);
    password_array
        .load(&directory_name[1], &directory_name[0])
        .unwrap();
    loop {
        action(
            menu(&items, prompt).try_into().unwrap(),
            &mut password_array,
            &directory_name[0],
        );
    }
}
