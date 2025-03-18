use argon2::{
    Algorithm, Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::engine::general_purpose::URL_SAFE;
use fernet::Fernet;
use secrecy::SecretString;

const MASTER_LENGTH: usize = 64;
const MEMORY_COST: u32 = 2_u32.pow(16);
const T_COST: u32 = 3;
const P_COST: u32 = 1;

const KEY_LENGTH: usize = 32;
const MEMORY_COST_KEY: u32 = 2_u32.pow(16);
const T_COST_KEY: u32 = 3;
const P_COST_KEY: u32 = 1;

const SALT_LENGTH: usize = 32;

/// Hashes using Argon2id
///
/// # Example
/// ```
/// use nspm::cryptography::{hash, generate_salt};
/// use argon2::password_hash::SaltString;
/// let salt = SaltString::from_b64("677DhCspdGNHgyuHm+R3+5NU/0MRYDDw6AfgdPLMXeY").unwrap();
/// let hashed = hash(b"paper", &salt).unwrap();
/// ```
pub fn hash(string: &[u8], salt: &SaltString) -> Result<String, argon2::password_hash::Error> {
    let argon = Argon2::new(
        Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(MEMORY_COST, T_COST, P_COST, Some(MASTER_LENGTH))?,
    );
    let hash_output = argon.hash_password(string, salt)?.to_string();
    Ok(URL_SAFE.encode(hash_output))
}

/// Checks if `hash_check` is equal to the hash of string (with salt)
/// # Example
/// ```
/// use nspm::cryptography::check_hash;
/// use nspm::cryptography::{hash, generate_salt};
/// use argon2::password_hash::SaltString;
/// let salt = SaltString::from_b64("677DhCspdGNHgyuHm+R3+5NU/0MRYDDw6AfgdPLMXeY").unwrap();
/// let hashed = hash(b"paper", &salt).unwrap();
/// assert!(check_hash("paper", &hashed))
/// ```
pub fn check_hash(string: &str, hash_check: &str) -> bool {
    let hash_check_decoded = String::from_utf8(URL_SAFE.decode(hash_check).unwrap()).unwrap();
    let parsed_hash = PasswordHash::new(&hash_check_decoded).unwrap();
    Argon2::new(
        Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(MEMORY_COST, T_COST, P_COST, Some(MASTER_LENGTH)).unwrap(),
    )
    .verify_password(string.as_bytes(), &parsed_hash)
    .is_ok()
}

/// Generates a salt of length 32 (trust me i need this)
pub fn generate_salt(
    rand: &mut impl rand_core::TryCryptoRng,
) -> Result<SaltString, argon2::password_hash::Error> {
    let mut buffer = [0u8; SALT_LENGTH];
    rand.try_fill_bytes(&mut buffer).unwrap();
    SaltString::from_b64(&STANDARD_NO_PAD.encode(buffer))
}

/// Encrypts `pwd` with `master_pwd` using fernet encryption
///
/// # Example
/// ```
/// use nspm::cryptography::encrypt;
/// use argon2::password_hash::SaltString;
/// let salt = SaltString::from_b64("/NQctu0+XVTdWle/+JlMdT2lE+wIxELEHqIBebsypek").unwrap();
/// let master = b"p";
/// encrypt(b"p", master, salt);
/// ```
pub fn encrypt(pwd: &[u8], master_pwd: &[u8], salt: SaltString) -> String {
    let mut key = [0u8; KEY_LENGTH];
    let buffer = pwd;
    let argon = Argon2::new_with_secret(
        master_pwd,
        Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(MEMORY_COST_KEY, T_COST_KEY, P_COST_KEY, Some(KEY_LENGTH)).unwrap(),
    )
    .unwrap();
    let _ = argon.hash_password_into(master_pwd, salt.as_str().as_bytes(), &mut key);
    let key_b64 = URL_SAFE.encode(key);
    let f = Fernet::new(key_b64.as_str()).unwrap();
    f.encrypt(buffer)
}

/// Decrypts pwd using master_pwd
///
/// # Example:
/// ```
/// use nspm::cryptography::decrypt;
/// use nspm::cryptography::encrypt;
/// use secrecy::SecretString;
/// use argon2::password_hash::SaltString;
/// let salt = SaltString::from_b64("/NQctu0+XVTdWle/+JlMdT2lE+wIxELEHqIBebsypek").unwrap();
/// let master = b"p";
/// let fernet_encrypted = encrypt(b"p", master, salt.clone());
/// decrypt(fernet_encrypted.as_bytes(), master, salt);
/// ```
///
/// # Panics
///
/// Panics if master_pwd is not correct.
pub fn decrypt(pwd: &[u8], master_pwd: &[u8], salt: SaltString) -> Result<SecretString, String> {
    let mut key = [0u8; KEY_LENGTH];
    let buffer = pwd;
    let argon = Argon2::new_with_secret(
        master_pwd,
        Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(MEMORY_COST_KEY, T_COST_KEY, P_COST_KEY, Some(KEY_LENGTH)).unwrap(),
    )
    .unwrap();
    let _ = argon.hash_password_into(master_pwd, salt.as_str().as_bytes(), &mut key);
    let key_b64 = URL_SAFE.encode(key);
    let f = Fernet::new(key_b64.as_str()).unwrap();
    let buffer_str = std::str::from_utf8(buffer).unwrap();
    let decrypted = f
        .decrypt(buffer_str)
        .map_err(|e| format!("Error with decryption: {e}"))?;
    let decrypted_str = String::from_utf8(decrypted).unwrap();
    Ok(SecretString::from(decrypted_str))
}
