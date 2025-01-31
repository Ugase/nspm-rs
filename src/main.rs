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
    let directory_name = directory_selector();
    if directory_name[2] == "true".to_string() {
        let mut password_array = PasswordArray::new(directory_name[1].clone());
        loop {
            action(
                menu(&items).try_into().unwrap(),
                &mut password_array,
                directory_name[0].clone(),
            );
        }
    } else {
        let mut password_array = PasswordArray::new(directory_name[0].clone());
        password_array
            .load(directory_name[1].clone(), directory_name[0].clone())
            .unwrap();
        loop {
            action(
                menu(&items).try_into().unwrap(),
                &mut password_array,
                directory_name[0].clone(),
            );
        }
    }
}
