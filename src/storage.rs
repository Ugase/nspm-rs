use crate::cryptography::{decrypt, encrypt, generate_salt, hash};
use argon2::password_hash::SaltString;
use comfy_table::{
    ContentArrangement, Table,
    modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS},
    presets::UTF8_FULL_CONDENSED,
};
use rand_core::OsRng;
use std::fs;
use std::iter::zip;

#[derive(Debug, Clone)]
pub struct Password {
    service_name: String,
    password: String,
    salt: SaltString,
    master_password: String,
    is_encrypted: bool,
}

#[derive(Debug, Clone)]
pub struct PasswordArray {
    passwords: Vec<Password>,
    services: Vec<String>,
    master_password: String,
    table: Table,
}

impl PasswordArray {
    pub fn new(master_password: String) -> PasswordArray {
        PasswordArray {
            passwords: vec![],
            services: vec![],
            master_password,
            table: Table::new(),
        }
    }
    pub fn save(&mut self, directory_name: String) {
        self.encrypt();
        for (index, password) in zip(0..self.passwords.len(), self.passwords.clone()) {
            let password_location = format!("{directory_name}/passwords/password_{index}");
            let service_location = format!("{directory_name}/services/service_{index}");
            let salt_location = format!("{directory_name}/salts/salt_{index}");
            password.save(&password_location, &salt_location, &service_location);
        }
    }
    pub fn load(&mut self, master_password: String, directory_name: String) -> Result<(), &str> {
        self.master_password = master_password;
        if !self.passwords.is_empty() {
            return Err("self.passwords not empty");
        }
        let amount_of_passwords = fs::read_dir(format!("{directory_name}/passwords"))
            .unwrap()
            .count();
        for index in 0..amount_of_passwords {
            let service_name =
                fs::read_to_string(format!("{directory_name}/services/service_{index}")).unwrap();
            let encrypted_password =
                fs::read_to_string(format!("{directory_name}/passwords/password_{index}")).unwrap();
            let salt = SaltString::from_b64(
                fs::read_to_string(format!("{directory_name}/salts/salt_{index}"))
                    .unwrap()
                    .as_str(),
            )
            .unwrap();
            let password = Password {
                service_name: service_name.clone(),
                password: encrypted_password,
                salt,
                master_password: self.master_password.clone(),
                is_encrypted: true,
            };
            self.passwords.push(password);
            self.services.push(service_name);
        }
        self.decrypt();
        self.update_table();
        Ok(())
    }
    pub fn add_password(&mut self, service: String, password: String) -> Result<(), String> {
        if self.services.contains(&service) {
            return Err("service name is taken".to_string());
        }
        self.passwords.push(Password::new(
            service.clone(),
            password,
            self.master_password.clone(),
        ));
        self.services.push(service);
        self.update_table();
        Ok(())
    }
    fn find_index(&self, service_name: String) -> Option<usize> {
        for (index, service) in zip(0..self.services.len(), &self.services) {
            if *service == service_name {
                return Some(index);
            }
        }
        None
    }
    pub fn edit_password(&mut self, service_name: String, new_pass: String) -> Result<(), String> {
        if !self.services.contains(&service_name) {
            return Err("service does not exist".to_string());
        }
        let index = self.find_index(service_name).unwrap();
        let a: &mut Password = self.passwords.get_mut(index).unwrap();
        let _ = a.edit_password(new_pass);
        self.update_table();
        Ok(())
    }
    pub fn remove_password(&mut self, service_name: String) -> Result<(), String> {
        let index = self.find_index(service_name);
        if index.is_none() {
            return Err(String::from("couldn't find service"));
        }
        let index = index.unwrap();
        self.passwords.remove(index);
        self.services.remove(index);
        self.update_table();
        Ok(())
    }
    fn decrypt(&mut self) {
        for password in self.passwords.iter_mut() {
            let _ = password.decrypt();
        }
    }
    fn encrypt(&mut self) {
        for password in self.passwords.iter_mut() {
            password.encrypt();
        }
    }
    fn update_table(&mut self) {
        self.table = Table::new();
        let mut passwords = vec![];
        for password in self.passwords.iter() {
            passwords.push(password.password.clone());
        }
        let mut result = vec![];
        for (service, password) in zip(self.services.clone(), passwords) {
            result.push(vec![service, password]);
        }
        self.table
            .load_preset(UTF8_FULL_CONDENSED)
            .apply_modifier(UTF8_SOLID_INNER_BORDERS)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_width(50)
            .set_header(vec!["Services", "Passwords"])
            .add_rows(result);
    }
    pub fn table(&self) -> Table {
        self.table.clone()
    }
}

impl Password {
    pub fn new(service_name: String, password: String, master: String) -> Password {
        Password {
            service_name,
            password,
            salt: generate_salt(&mut OsRng).unwrap(),
            master_password: master,
            is_encrypted: false,
        }
    }
    pub fn save(self, file_name: &str, salt_location: &str, service_location: &str) {
        if !self.is_encrypted {
            panic!("not encrypted");
        }
        fs::write(file_name, self.password).unwrap();
        fs::write(salt_location, self.salt.as_str()).unwrap();
        fs::write(service_location, self.service_name).unwrap();
    }
    pub fn encrypt(&mut self) {
        self.password = encrypt(
            self.password.as_bytes(),
            self.master_password.as_bytes(),
            self.salt.clone(),
        );
        self.master_password = String::new();
        self.is_encrypted = true;
    }
    pub fn decrypt(&mut self) -> Result<(), String> {
        if !self.is_encrypted {
            return Err("already decrypted".to_string());
        }
        self.password = decrypt(
            self.password.as_bytes(),
            self.master_password.as_bytes(),
            self.salt.clone(),
        );
        self.is_encrypted = false;
        Ok(())
    }
    fn edit_password(&mut self, new_pass: String) -> Result<(), String> {
        if self.is_encrypted {
            return Err("is encrypted".to_string());
        }
        self.password = new_pass;
        Ok(())
    }
}

fn create_master_password(master_password: &String, dir_name: &String) {
    let salt = generate_salt(&mut OsRng).unwrap();
    let _ = fs::write(
        dir_name.to_owned() + &String::from("/master_password"),
        hash(master_password.as_bytes(), salt.as_str().as_bytes()).unwrap(),
    );
    let _ = fs::write(
        dir_name.to_owned() + &String::from("/master_password_salt"),
        salt.as_str(),
    );
}

pub fn get_master_password(dir_name: &String) -> Result<[String; 2], std::io::Error> {
    let master_password =
        fs::read_to_string(dir_name.to_owned() + &String::from("/master_password"))?;
    let salt = fs::read_to_string(dir_name.to_owned() + &String::from("/master_password_salt"))?;
    Ok([master_password, salt])
}

pub fn initialize_directory(name: &String, master_password: &String) {
    let _ = fs::create_dir(name);
    let _ = fs::create_dir(name.to_owned() + &String::from("/passwords"));
    let _ = fs::create_dir(name.to_owned() + &String::from("/services"));
    let _ = fs::create_dir(name.to_owned() + &String::from("/salts"));
    create_master_password(master_password, name);
}

pub fn verify_directory(dir_name: &String) -> bool {
    let list: [String; 5] = [
        String::from("salts"),
        String::from("passwords"),
        String::from("master_password"),
        String::from("master_password_salt"),
        String::from("services"),
    ];
    for file in list {
        if !fs::exists(format!("{}/{}", dir_name, file)).unwrap() {
            return false;
        }
    }
    true
}
