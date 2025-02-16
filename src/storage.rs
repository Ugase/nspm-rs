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

/// A password with service and salt metadata
#[derive(Debug, Clone)]
pub struct Password {
    service: String,
    password: String,
    salt: SaltString,
    key: String,
    is_encrypted: bool,
}

/// An array of [`Password`] that's better than an array of [`Password`]
#[derive(Debug, Clone)]
pub struct PasswordArray {
    passwords: Vec<Password>,
    master_password: String,
    pub table: Table,
}

impl PasswordArray {
    /// Makes a new empty [`PasswordArray`] with master_password
    pub fn new(master_password: &str) -> PasswordArray {
        PasswordArray {
            passwords: vec![],
            master_password: master_password.to_string(),
            table: Table::new(),
        }
    }
    /// Saves all passwords in a directory that can be loaded with [load][PasswordArray::load]
    ///
    /// # Panics
    /// Panics if directory doesn't exist
    pub fn save(&mut self, directory_name: &str) {
        self.encrypt();
        for (index, password) in zip(0..self.passwords.len(), self.passwords.clone()) {
            let password_location = format!("{directory_name}/passwords/password_{index}");
            let service_location = format!("{directory_name}/services/service_{index}");
            let salt_location = format!("{directory_name}/salts/salt_{index}");
            password.save(&password_location, &salt_location, &service_location);
        }
    }
    /// Loads a directory to a [PasswordArray]
    ///
    /// # Panics
    /// Panics when a file or directory doesn't exist or master password is wrong
    pub fn load(&mut self, master_password: &str, directory_name: &str) -> Result<(), &str> {
        self.master_password = master_password.to_string();
        if !self.passwords.is_empty() {
            return Err("self.passwords not empty");
        }
        let amount_of_passwords = fs::read_dir(format!("{directory_name}/passwords"))
            .unwrap()
            .count();
        for index in 0..amount_of_passwords {
            self.passwords.push(Password::load(
                format!("{directory_name}/passwords/password_{index}").as_str(),
                format!("{directory_name}/salts/salt_{index}").as_str(),
                format!("{directory_name}/services/service_{index}").as_str(),
                self.master_password.as_str(),
            ));
        }
        self.decrypt();
        self.update_table();
        Ok(())
    }
    /// Adds a password to [PasswordArray]
    pub fn add_password(&mut self, service: String, password: String) -> Result<(), &str> {
        if self.get_services().contains(&service) {
            return Err("service name is taken");
        }
        self.passwords.push(Password::new(
            service,
            password,
            self.master_password.clone(),
        ));
        self.update_table();
        Ok(())
    }
    fn find_index(&self, service_name: String) -> Option<usize> {
        let services = self.get_services();
        for (index, service) in zip(0..services.len(), services) {
            if service == service_name {
                return Some(index);
            }
        }
        None
    }
    /// (hopefully self explanatory)
    pub fn edit_password(&mut self, service_name: String, new_pass: String) -> Result<(), &str> {
        if !self.get_services().contains(&service_name) {
            return Err("password does not exist");
        }
        let index: usize = self.find_index(service_name).unwrap();
        let a: &mut Password = self.passwords.get_mut(index).unwrap();
        let _ = a.edit_password(new_pass);
        self.update_table();
        Ok(())
    }
    /// (guess)
    pub fn remove_password(&mut self, service_name: String) -> Result<(), &str> {
        let index = self.find_index(service_name);
        if index.is_none() {
            return Err("couldn't find service");
        }
        let index = index.unwrap();
        self.passwords.remove(index);
        self.update_table();
        Ok(())
    }
    fn decrypt(&mut self) {
        for password in self.passwords.iter_mut() {
            password.decrypt().unwrap();
        }
    }
    fn encrypt(&mut self) {
        for password in self.passwords.iter_mut() {
            password.encrypt().unwrap();
        }
    }
    fn update_table(&mut self) {
        self.table = Table::new();
        let mut passwords = vec![];
        for password in self.passwords.iter() {
            passwords.push(password.password.clone());
        }
        let mut result = vec![];
        for (service, password) in zip(self.get_services(), passwords) {
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
    fn get_services(&self) -> Vec<String> {
        let mut res = vec![];
        for password in self.passwords.clone() {
            res.push(password.service);
        }
        res
    }
}

impl Password {
    /// creates a new password with a randomly generated salt
    pub fn new(service_name: String, password: String, master: String) -> Password {
        Password {
            service: service_name,
            password,
            salt: generate_salt(&mut OsRng).unwrap(),
            key: master,
            is_encrypted: false,
        }
    }
    /// saves the [Password] details to 3 different files
    pub fn save(self, password_location: &str, salt_location: &str, service_location: &str) {
        if !self.is_encrypted {
            panic!("not encrypted");
        }
        fs::write(password_location, self.password).unwrap();
        fs::write(salt_location, self.salt.as_str()).unwrap();
        fs::write(service_location, self.service).unwrap();
    }
    /// Makes encrypted [Password] from the 3 files that [Password::save] writes
    /// assumes that the password is encrypted
    ///
    /// # Panics
    /// Panics if either one of the file locations doesn't exist or if the salt stored at salt
    /// location is in base64
    pub fn load(
        password_location: &str,
        salt_location: &str,
        service_location: &str,
        master_password: &str,
    ) -> Password {
        Password {
            password: fs::read_to_string(password_location).unwrap(),
            salt: SaltString::from_b64(&fs::read_to_string(salt_location).unwrap())
                .expect("salt not in base64"),
            service: fs::read_to_string(service_location).unwrap(),
            key: master_password.to_string(),
            is_encrypted: true,
        }
    }
    /// encrypts the password with key (doesn't encrypt when already encrypted)
    /// also throws away the key
    pub fn encrypt(&mut self) -> Result<(), &str> {
        if self.is_encrypted {
            return Err("already encrypted");
        }
        self.password = encrypt(
            self.password.as_bytes(),
            self.key.as_bytes(),
            self.salt.clone(),
        );
        self.key = String::new();
        self.is_encrypted = true;
        Ok(())
    }
    /// decrypts the password using the key
    /// Fails if [Password]'s password is already decrypted or when the key is empty
    pub fn decrypt(&mut self) -> Result<(), String> {
        if !self.is_encrypted {
            return Err("already decrypted".to_string());
        }
        if self.key.is_empty() {
            return Err("no key".to_string());
        }
        self.password = decrypt(
            self.password.as_bytes(),
            self.key.as_bytes(),
            self.salt.clone(),
        )?;
        self.is_encrypted = false;
        Ok(())
    }
    fn edit_password(&mut self, new_pass: String) -> Result<(), &str> {
        if self.is_encrypted {
            return Err("is encrypted");
        }
        self.password = new_pass;
        Ok(())
    }
}

fn create_master_password(master_password: &str, dir_name: &str) {
    let salt = generate_salt(&mut OsRng).unwrap();
    let _ = fs::write(
        format!("{dir_name}/master_password"),
        hash(master_password.as_bytes(), salt.as_str().as_bytes()).unwrap(),
    );
    let _ = fs::write(format!("{dir_name}/master_password_salt"), salt.as_str());
}

/// Gets master password and its salt the directory must exist and is vaild for this function to
/// work
pub fn get_master_password(dir_name: &str) -> Result<[String; 2], std::io::Error> {
    let master_password = fs::read_to_string(format!("{dir_name}/master_password"))?;
    let salt = fs::read_to_string(format!("{dir_name}/master_password_salt"))?;
    Ok([master_password, salt])
}

/// Initialize's the directories and makes the master password
/// its used to create new directories for the password manager to manage
pub fn initialize_directory(name: &str, master_password: &str) {
    let _ = fs::create_dir(name);
    let _ = fs::create_dir(format!("{name}/passwords"));
    let _ = fs::create_dir(format!("{name}/services"));
    let _ = fs::create_dir(format!("{name}/salts"));
    create_master_password(master_password, name);
}

/// Checks if the directory and the correct files and directories exists
pub fn verify_directory(dir_name: &str) -> bool {
    let list: [String; 2] = [
        format!("{dir_name}/master_password"),
        format!("{dir_name}/master_password_salt"),
    ];
    let dirs = [
        format!("{dir_name}/salts"),
        format!("{dir_name}/passwords"),
        format!("{dir_name}/services"),
    ];
    for file in list {
        if !fs::exists(file).unwrap() {
            return false;
        }
    }
    for dir in dirs {
        let meta = fs::metadata(dir);
        if meta.is_err() {
            return false;
        }
        if !meta.unwrap().is_dir() {
            return false;
        }
    }
    true
}
