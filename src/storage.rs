use crate::{
    ansi::{Csi, EL},
    cryptography::{decrypt, encrypt, generate_salt, hash},
    ui::ProgressBar,
};
use argon2::password_hash::SaltString;
use comfy_table::{ContentArrangement, Table};
use rand_core::OsRng;
use secrecy::{ExposeSecret, SecretString};
use std::{fs, io::Write, iter::zip, time::Duration};

/// A password with service and salt metadata
#[derive(Debug, Clone)]
pub struct Password {
    service: String,
    password: SecretString,
    salt: SaltString,
    key: SecretString,
    is_encrypted: bool,
}

/// An array of [`Password`] that's better than an array of [`Password`]
pub struct PasswordArray {
    passwords: Vec<Password>,
    master_password: SecretString,
    directory_name: String,
}

impl PasswordArray {
    /// Makes a new empty [`PasswordArray`] with master_password
    pub fn new(master_password: SecretString, directory_name: String) -> PasswordArray {
        PasswordArray {
            passwords: vec![],
            master_password,
            directory_name,
        }
    }
    /// Saves all passwords in a directory that can be loaded with [load][PasswordArray::load]
    ///
    /// # Panics
    /// Panics if directory doesn't exist
    pub fn save(&mut self, print_state: bool) {
        let mut progress = ProgressBar::new(self.passwords.len() as u32 * 3);
        let _ = fs::remove_dir_all(&self.directory_name);
        initialize_directory(&self.directory_name, self.master_password.expose_secret());
        self.encrypt(print_state, &mut progress);
        for (index, password) in self.passwords.iter().enumerate() {
            let password_location = format!("{}/passwords/password_{index}", self.directory_name);
            let service_location = format!("{}/services/service_{index}", self.directory_name);
            let salt_location = format!("{}/salts/salt_{index}", self.directory_name);
            if print_state {
                let mut buf = std::io::stdout();
                progress.increse_n();
                print!(
                    "{}{}{}",
                    Csi::CPL.ansi(),
                    Csi::CNL.ansi(),
                    Csi::El(EL::EL2).ansi()
                );
                let _ = buf.flush();
                print!("{} Saving, {}", progress, password.service);
                let _ = buf.flush();
                sleep(45);
            }
            password.save(&password_location, &salt_location, &service_location);
        }
        println!();
    }
    /// Loads a directory to a [PasswordArray]
    ///
    /// # Panics
    /// Panics when a file or directory doesn't exist or master password is wrong
    pub fn load(&mut self, directory_name: &str, print_state: bool) -> Result<(), &str> {
        if !self.passwords.is_empty() {
            return Err("self.passwords not empty");
        }
        let amount_of_passwords: usize = fs::read_dir(format!("{directory_name}/passwords"))
            .unwrap()
            .count();
        let mut progress = ProgressBar::new(amount_of_passwords as u32 * 3);
        self.directory_name = directory_name.to_string();
        for index in 0..amount_of_passwords {
            self.passwords.push(Password::load(
                format!("{directory_name}/passwords/password_{index}").as_str(),
                format!("{directory_name}/salts/salt_{index}").as_str(),
                format!("{directory_name}/services/service_{index}").as_str(),
                self.master_password.expose_secret(),
            ));
            if print_state {
                let mut buf = std::io::stdout();
                print!(
                    "{}{}{}",
                    Csi::CPL.ansi(),
                    Csi::CNL.ansi(),
                    Csi::El(EL::EL2).ansi()
                );
                let _ = buf.flush();
                progress.increse_n();
                print!(
                    "{} Loaded, {}",
                    progress,
                    self.passwords.get(index).unwrap().service
                );
                let _ = buf.flush();
                sleep(45);
            }
        }
        self.decrypt(print_state, &mut progress);
        println!();
        Ok(())
    }
    /// Adds a password to [PasswordArray]
    pub fn add_password(&mut self, service: String, password: SecretString) -> Result<(), &str> {
        if self.get_services().contains(&service) {
            return Err("service name is taken");
        }
        self.passwords.push(Password::new(
            service,
            password,
            self.master_password.clone(),
        ));
        Ok(())
    }
    /// (hopefully self explanatory)
    pub fn edit_password(
        &mut self,
        service_name: String,
        new_pass: SecretString,
    ) -> Result<(), &str> {
        let index = self.get_services().iter().position(|s| *s == service_name);
        if index.is_none() {
            return Err("couldn't find service");
        }
        let index = index.unwrap();
        let a: &mut Password = self.passwords.get_mut(index).unwrap();
        let _ = a.edit_password(new_pass);
        Ok(())
    }
    /// (guess)
    pub fn remove_password(&mut self, service_name: String) -> Result<(), &str> {
        let index = self.get_services().iter().position(|s| *s == service_name);
        if index.is_none() {
            return Err("couldn't find service");
        }
        let index = index.unwrap();
        self.passwords.remove(index);
        Ok(())
    }
    fn decrypt(&mut self, print_state: bool, progress_bar: &mut ProgressBar) {
        for password in self.passwords.iter_mut() {
            if print_state {
                let mut buf = std::io::stdout();
                print!(
                    "{}{}{}",
                    Csi::CPL.ansi(),
                    Csi::CNL.ansi(),
                    Csi::El(EL::EL2).ansi()
                );
                let _ = buf.flush();
                progress_bar.increse_n();
                print!("{} Decrypting, {}", progress_bar, password.service);
                let _ = buf.flush();
            }
            password.decrypt().unwrap();
            if print_state {
                let mut buf = std::io::stdout();
                print!(
                    "{}{}{}",
                    Csi::CPL.ansi(),
                    Csi::CNL.ansi(),
                    Csi::El(EL::EL2).ansi()
                );
                let _ = buf.flush();
                progress_bar.increse_n();
                print!("{} Decrypted, {}", progress_bar, password.service);
                let _ = buf.flush();
            }
        }
    }
    fn encrypt(&mut self, print_state: bool, progress_bar: &mut ProgressBar) {
        for password in self.passwords.iter_mut() {
            if print_state {
                let mut buf = std::io::stdout();
                print!(
                    "{}{}{}",
                    Csi::CPL.ansi(),
                    Csi::CNL.ansi(),
                    Csi::El(EL::EL2).ansi()
                );
                let _ = buf.flush();
                progress_bar.increse_n();
                print!("{} Encrypting, {}", progress_bar, password.service);
                let _ = buf.flush();
            }
            password.encrypt().unwrap();
            if print_state {
                let mut buf = std::io::stdout();
                print!(
                    "{}{}{}",
                    Csi::CPL.ansi(),
                    Csi::CNL.ansi(),
                    Csi::El(EL::EL2).ansi()
                );
                let _ = buf.flush();
                progress_bar.increse_n();
                print!("{} Encrypted, {}", progress_bar, password.service);
                let _ = buf.flush();
            }
        }
    }
    pub fn table(&mut self) -> Table {
        let mut passwords = vec![];
        for password in self.passwords.iter() {
            passwords.push(password.password.expose_secret());
        }
        let mut result = vec![];
        for (service, password) in zip(self.get_services(), passwords) {
            result.push(vec![service, password.to_string()]);
        }
        let mut tables = Table::new();
        tables
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Services", "Passwords"])
            .add_rows(result);
        tables
    }
    pub fn get_services(&self) -> Vec<String> {
        self.passwords.iter().map(|p| p.service.clone()).collect()
    }
}

impl Password {
    /// creates a new password with a randomly generated salt
    pub fn new(service_name: String, password: SecretString, master: SecretString) -> Password {
        Password {
            service: service_name,
            password,
            salt: generate_salt(&mut OsRng).unwrap(),
            key: master,
            is_encrypted: false,
        }
    }
    /// saves [Password]'s details to 3 different files
    pub fn save(&self, password_location: &str, salt_location: &str, service_location: &str) {
        if !self.is_encrypted {
            panic!("not encrypted");
        }
        fs::write(password_location, self.password.expose_secret()).unwrap();
        fs::write(salt_location, self.salt.as_str()).unwrap();
        fs::write(service_location, &self.service).unwrap();
    }
    /// Makes encrypted [Password] from the 3 files that [Password::save] writes
    /// assumes that the password is encrypted
    ///
    /// # Panics
    /// Panics if either one of the file locations doesn't exist or if the salt stored at salt
    /// location is not in base64
    pub fn load(
        password_location: &str,
        salt_location: &str,
        service_location: &str,
        master_password: &str,
    ) -> Password {
        Password {
            password: SecretString::from(fs::read_to_string(password_location).unwrap()),
            salt: SaltString::from_b64(&fs::read_to_string(salt_location).unwrap())
                .expect("salt not in base64"),
            service: fs::read_to_string(service_location).unwrap(),
            key: SecretString::from(master_password),
            is_encrypted: true,
        }
    }
    /// encrypts the password with key (doesn't encrypt when already encrypted)
    /// also throws away the key
    pub fn encrypt(&mut self) -> Result<(), &str> {
        if self.is_encrypted {
            return Err("already encrypted");
        }
        self.password = SecretString::from(encrypt(
            self.password.expose_secret().as_bytes(),
            self.key.expose_secret().as_bytes(),
            self.salt.clone(),
        ));
        self.key = SecretString::from("");
        self.is_encrypted = true;
        Ok(())
    }
    /// decrypts the password using the key
    /// Fails if [Password]'s password is already decrypted or when the key is empty
    pub fn decrypt(&mut self) -> Result<(), String> {
        if !self.is_encrypted {
            return Err("already decrypted".to_string());
        }
        if self.key.expose_secret().is_empty() {
            return Err("no key".to_string());
        }
        self.password = decrypt(
            self.password.expose_secret().as_bytes(),
            self.key.expose_secret().as_bytes(),
            self.salt.clone(),
        )?;
        self.is_encrypted = false;
        Ok(())
    }
    fn edit_password(&mut self, new_pass: SecretString) -> Result<(), &str> {
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
        hash(master_password.as_bytes(), &salt).unwrap(),
    );
}

/// Gets master password and its salt the directory must exist and is valid for this function to
/// work
pub fn get_master_password(dir_name: &str) -> Result<String, std::io::Error> {
    let master_password = fs::read_to_string(format!("{dir_name}/master_password"))?;
    Ok(master_password)
}

/// Initializes the directories and makes the master password
/// its used to create new directories for the password manager to manage
pub fn initialize_directory(name: &str, master_password: &str) {
    fs::create_dir(name).unwrap();
    fs::create_dir(format!("{name}/passwords")).unwrap();
    fs::create_dir(format!("{name}/services")).unwrap();
    fs::create_dir(format!("{name}/salts")).unwrap();
    create_master_password(master_password, name);
}

/// Checks if the directory and the correct files and directories exists
pub fn verify_directory(dir_name: &str) -> bool {
    let list: [String; 1] = [format!("{dir_name}/master_password")];
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

fn sleep(duration_millis: u64) {
    std::thread::sleep(Duration::from_millis(duration_millis));
}
