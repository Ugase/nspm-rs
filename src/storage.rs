use crate::{
    ansi::clear_line,
    cryptography::{decrypt, encrypt, generate_salt, hash},
    ui::{NO_COMMANDS, NO_FLAGS, ProgressBar, YESES, input},
};
use argon2::password_hash::SaltString;
use comfy_table::{ContentArrangement, Table};
use rand_core::OsRng;
use secrecy::{ExposeSecret, SecretString};
use std::{
    fs,
    io::{Write, stdout},
    iter::zip,
    process::exit,
    time::Duration,
};

/// A password with service and salt metadata
#[derive(Debug, Clone)]
pub struct Password {
    service: String,
    password: SecretString,
    salt: SaltString,
    key: SecretString,
    is_encrypted: bool,
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
    pub fn save(
        &self,
        password_location: &str,
        salt_location: &str,
        service_location: &str,
    ) -> Result<(), String> {
        if !self.is_encrypted {
            panic!("not encrypted");
        }
        fs::write(password_location, self.password.expose_secret())
            .map_err(|err| format!("Error when writing password: {err}"))?;
        fs::write(salt_location, self.salt.as_str())
            .map_err(|err| format!("Error when writing salt: {err}"))?;
        fs::write(service_location, &self.service)
            .map_err(|err| format!("Error when writing service: {err}"))?;
        Ok(())
    }
    /// Makes encrypted [Password] from the 3 files that [Password::save] writes
    /// Assumes that the password is encrypted
    ///
    /// # Panics
    /// Panics if either one of the file locations doesn't exist or if the salt stored at salt
    /// location is not in base64
    pub fn load(
        password_location: &str,
        salt_location: &str,
        service_location: &str,
        master_password: &str,
    ) -> Result<Password, String> {
        Ok(Password {
            password: SecretString::from(
                fs::read_to_string(password_location)
                    .map_err(|err| format!("Failed to read: {password_location}, Error: {err}"))?,
            ),
            salt: SaltString::from_b64(
                &fs::read_to_string(salt_location)
                    .map_err(|err| format!("Failed to read {salt_location}, Error: {err}"))?,
            )
            .map_err(|err| format!("Failed to decode from base64: {err}"))?,
            service: fs::read_to_string(service_location)
                .map_err(|err| format!("Failed to read {service_location}: {err}"))?,
            key: SecretString::from(master_password),
            is_encrypted: true,
        })
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

/// An array of [`Password`] that's better than an array of [`Password`]
pub struct PasswordArray {
    passwords: Vec<Password>,
    master_password: SecretString,
    directory_name: String,
}

impl PasswordArray {
    /// Makes a new empty [`PasswordArray`] with master_password and directory_name
    pub fn new(master_password: SecretString, directory_name: String) -> PasswordArray {
        PasswordArray {
            passwords: vec![],
            master_password,
            directory_name,
        }
    }
    /// Saves all passwords in a directory that can be loaded with [load][PasswordArray::load]
    pub fn save(&mut self, print_progress_bar: bool) -> Result<(), String> {
        let temporary_directory: String = format!("{}_tmp", self.directory_name);
        let mut progress_bar = ProgressBar::new((self.passwords.len() as u32 * 3) + 4);
        if fs::exists(&temporary_directory).map_err(|err| {
            format!("Error when checking if temporary directory already exists: {err}")
        })? {
            let yn = input(
                "Temporary directory already exists remove it (Y/n)? ",
                "y".to_string(),
                NO_COMMANDS,
                NO_FLAGS,
            );
            if !YESES.contains(&&yn.to_lowercase()[..]) {
                exit(0)
            }
            fs::remove_dir_all(&temporary_directory).map_err(|err| {
                format!("Error when removing existing temporary directory: {err}")
            })?;
        }
        if print_progress_bar {
            progress_bar.increase_n();
            clear_line();
            simpler_print(format!("{progress_bar} Making temporary directory"));
        }
        initialize_directory(&temporary_directory, self.master_password.expose_secret());
        if print_progress_bar {
            progress_bar.increase_n();
            clear_line();
            simpler_print(format!("{progress_bar} Made temporary directory"));
        }
        self.encrypt(print_progress_bar, &mut progress_bar);
        for (index, password) in self.passwords.iter().enumerate() {
            let password_location = format!("{temporary_directory}/passwords/password_{index}");
            let service_location = format!("{temporary_directory}/services/service_{index}");
            let salt_location = format!("{temporary_directory}/salts/salt_{index}");
            if print_progress_bar {
                progress_bar.increase_n();
                clear_line();
                simpler_print(format!("{progress_bar} Saving, {}", password.service));
                sleep(45);
            }
            password.save(&password_location, &salt_location, &service_location)?;
        }
        if print_progress_bar {
            progress_bar.increase_n();
            clear_line();
            simpler_print(format!("{progress_bar} Moving temporary directory"));
        }
        fs::remove_dir_all(&self.directory_name)
            .map_err(|err| format!("Error with removing old directory: {err}"))?;
        fs::rename(&temporary_directory, &self.directory_name)
            .map_err(|err| format!("Error when moving directory: {err}"))?;
        if print_progress_bar {
            progress_bar.increase_n();
            clear_line();
            simpler_print(format!("{progress_bar} Moved temporary directory"));
        }
        println!();
        Ok(())
    }
    /// Loads a directory to a [PasswordArray]
    pub fn load(&mut self, print_progress_bar: bool) -> Result<(), String> {
        if !self.passwords.is_empty() {
            return Err(String::from("self.passwords not empty"));
        } else if !verify_directory(&self.directory_name) {
            return Err(String::from(
                "directory either doesn't exist or doesn't have the correct files and directories",
            ));
        }
        let amount_of_passwords: usize = fs::read_dir(format!("{}/passwords", self.directory_name))
            .unwrap()
            .count();
        let mut progress_bar = ProgressBar::new(amount_of_passwords as u32 * 3);
        for index in 0..amount_of_passwords {
            self.passwords.push(Password::load(
                format!("{}/passwords/password_{index}", self.directory_name).as_str(),
                format!("{}/salts/salt_{index}", self.directory_name).as_str(),
                format!("{}/services/service_{index}", self.directory_name).as_str(),
                self.master_password.expose_secret(),
            )?);
            if print_progress_bar {
                progress_bar.increase_n();
                clear_line();
                simpler_print(format!(
                    "{progress_bar} Loaded, {}",
                    self.passwords.get(index).unwrap().service
                ));
                sleep(45);
            }
        }
        self.decrypt(print_progress_bar, &mut progress_bar)?;
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
    fn decrypt(
        &mut self,
        print_progress_bar: bool,
        progress_bar: &mut ProgressBar,
    ) -> Result<(), String> {
        for password in self.passwords.iter_mut() {
            if print_progress_bar {
                progress_bar.increase_n();
                clear_line();
                simpler_print(format!("{progress_bar} Decrypting, {}", password.service));
            }
            password.decrypt()?;
            if print_progress_bar {
                progress_bar.increase_n();
                clear_line();
                simpler_print(format!("{progress_bar} Decrypted, {}", password.service));
            }
        }
        Ok(())
    }
    fn encrypt(&mut self, print_progress_bar: bool, progress_bar: &mut ProgressBar) {
        for password in self.passwords.iter_mut() {
            if print_progress_bar {
                progress_bar.increase_n();
                clear_line();
                simpler_print(format!("{progress_bar} Encrypting, {}", password.service));
            }
            password.encrypt().unwrap();
            if print_progress_bar {
                progress_bar.increase_n();
                clear_line();
                simpler_print(format!("{progress_bar} Encrypted, {}", password.service));
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

pub fn simpler_print(data: String) {
    let mut buf = stdout();
    print!("{data}");
    let _ = buf.flush();
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
    for dir in dirs.iter() {
        let meta = fs::metadata(dir);
        if meta.is_err() {
            return false;
        } else if !meta.unwrap().is_dir() {
            return false;
        }
    }
    if !(fs::read_dir(dirs[0].clone()).unwrap().count()
        == fs::read_dir(dirs[1].clone()).unwrap().count()
        && fs::read_dir(dirs[1].clone()).unwrap().count()
            == fs::read_dir(dirs[2].clone()).unwrap().count())
    {
        return false;
    }
    true
}

fn create_master_password(master_password: &str, dir_name: &str) {
    let salt = generate_salt(&mut OsRng).unwrap();
    let _ = fs::write(
        format!("{dir_name}/master_password"),
        hash(master_password.as_bytes(), &salt).unwrap(),
    );
}

fn sleep(duration_millis: u64) {
    std::thread::sleep(Duration::from_millis(duration_millis));
}
