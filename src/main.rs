use nspm::storage::PasswordArray;

fn main() {
    let dirname = String::from("haa");
    let master = String::from("here");
    let mut pa = PasswordArray::new(master.clone());
    pa.load(master, dirname.clone()).unwrap();
    pa.edit_password("haha".to_string(), "password".to_string());
    dbg!(&pa);
    pa.remove_password("haha".to_string());
    dbg!(pa);
}
