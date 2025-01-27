use argon2::password_hash::rand_core::OsRng;
use nspm::cryptography::{encrypt, generate_salt, hash};

fn main() {
    let s = generate_salt(&mut OsRng).unwrap();
    let a: &[u8; 3] = b"yay";
    let m = b"a";
    let a = hash(m, s.as_str().as_bytes()).unwrap();
    println!("{:?}", encrypt(m, a.as_str().as_bytes(), s))
}
