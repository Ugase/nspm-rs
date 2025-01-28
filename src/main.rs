use nspm::storage::PasswordArray;

fn main() {
    let dirname = String::from("haa");
    let (service_dir, password_dir, salt_dir) = (
        format!("{dirname}/serv"),
        format!("{dirname}/pass"),
        format!("{dirname}/salt"),
    );
    let master = String::from("here");
    let mut pa = PasswordArray::new(master);
    pa.add_password(String::from("here"), String::from("aa"));
    dbg!(pa);
}
