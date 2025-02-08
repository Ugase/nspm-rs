use argon2::{Algorithm, Argon2, password_hash::SaltString};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE;
use fernet::Fernet;

const SALT_LENGTH: usize = 32;
const KEY_LENGTH: usize = 32;

/// Hashes using Argon2id
///
/// # Example
/// ```
/// use nspm::cryptography::hash;
/// let master = b"paper";
/// let hashed = "c4-u-v5GHsNjDwjaQkHkuIr3I60NqGz617NDQcV_bMH4ux7lScqyFQAEOBTSh9rX-82iGnXI3KLORz4TXVPn1ShqCUIIGBD53kbydHHwg_b5JALSopbICmPwjp2XkZ-lF3NbUD2h8hPIaeHOp-iSje9qHH4IiQ8DvYdK3UYTNhh8qQYpm6KMMKHXr3a3XnnOz80tcG9Y0BxlSfusS992CfMiEj_d2yZNlUUMZrgLWyIwtF1sUbZo7jzvI9qaol2cNxElCFmEcEKjW5joElUFYdpR02QOp-cbLZVQU5V_BLghrmj73usCKoTv7FUu4InKBhsnjtKnU2q3Vlvqyvj0Ekf5wITvgEnj6U4RL6ZRPsr3aV1pvzasvEIfhYy7R3VihRH2qqUAttPEnEb9TK2KMHEEZfuLk7U1rrmdijl_jC97IgDDMcunib9fAeKpxsA1rfQALaAXswHwmPjLU-UJqfiJ3NiVHSFKyMeNJH-KdoqnNo9kfGsSocnp5gJtXgLld-OQzstBgcUBUxWH7k6EMJCuGe9O6OcDMgncwOJXqxAx15wloOn5l7aq6Db5bHEcznYpAPKf1c19AvuTv7euIDJ4YUxZfhD6LNPbdLvXHc05Yl8_bXPAsW3L-YogAGdpKTeLpD2jNJ-rwIe9OWh5jnsXVY6w-utUEddlv0ZuHtJeUBJKKHU90Dhlu7bOqLrQbEGt-YTfE_zx5lDSym7R2_yGRQp85NfkHYVvkLn6i2j7Fc3RVggT1mYPt0hDi3Nbz5wPDoe6m-kwnchRaap6ZVGfn6P8gpr5ky7Q43Thl26J_reO20TMLpbysLE1ZFZV4O-ONw76F5NUdQYJ26YAt026voua2OjBom1UCTtvkvhP04bOFWAQiFmBfOFYTkBZkus0eRZTFvx3Y6LcTHLtn4K45cEC7VUOkhdMHlgwrEkHLzFMqDPOXBe9f9GUaeY8l4jqBH0ybadsxrv_UIMjmsBf7TcpidMF6TgKVmb79sjEldAlwGb2Hnq9s2BhSrwdrqoTIHS7SzH6l5pk002X6XOUQ2Ct0NlJ4P6__Xy_MmDLbo3o_bDulIxoHppxBAfRFtAg3tUhJ8DTXFdTkmxPtzcy1YNQtdWgsTzEpB0rwc51GlxMN_1oQB2ooADpa7AHC-r-s5cFBzXW5cIlqcpj7lBgPr0xVYwmj9tWnjJ4GjiwKTDeCguCUPr5FPSJMpwmdRCrhIufGpc2iJBbpfbR1H4CEvkYfP_ZYPKLQgNgq1WEwZnhGHjx4IgPgfzIl8_JOlu0hb5M3p8W8yXL_n47SGD3RMqO5a9TDEquX7JbpZfp6oWbvgmG6Ki5tneGu49FXNsP6--diYmRJ4CNveCBmNXkAuYLHEIgTWCx1ol2dJJ-iej4vamgpekDx-Y5P06tnGM-FlaJtFx6VtmNqrFFsrAGv5i43s5stKzANlP9wM44JznZgDq_w2TOrmY2BFlKHHMqFizIrSRZNm0S6vsDEBowQfIXG7XcKe-CBJaOWK1K5eYjfoR4gM1Ifr9tx8w9nGM3vq5MGcRGa_dwwOl_ypNErgcc9f8D-9HBVeF3synbno_OTq3phmE2V4Qvtmmadikdoxv7dP5wsb9skvSNOI41ngkFjsRdUFU0-DKm6veddJjSqX51VO7VdMfODmjOc5lusNeelKhULB21a9sPKzocE18YiiF3GhHUKvbLbCXBcWY25w9lGSNXBcD6s-inL_gQ_pRLPON5ctXi8kEyaqcwsMucaf014fNP_aPBoWL20J7Uomzt2Pv-0NlPmoYsg-_a1paWwmR1SU4rQ_Jre4SFaK4XkkeveI2vUvdjS2aXrkkCHJ8Pk00D4HZ0aY7tgZM0Ln9_EibV9gbkKZQRW3OzocL1GDuM3pUrBkzJWC-_n4MzER8npMQKyZ3Obbit8vJTPX-bnbw8hJm83ufo1DM9j0llTAkTcy9jg7eOrvpPv-xqDRYt_sOULwwEAvYPjrGJv1cPkGBFYo8FPYbk701INJTw1loewmjqUX0W87iYIl1eAvcqg_mwlgZ188sU9urIcgbw8ZR3DX5328YH3IBtEe7KDThaE4qNuGtN_YFUf6TS9j3IOQ5OLG06yrn0PM63T2XSl8GzE_iFiC9xqZQq3CWDWmUf5AAOQuW9zdfCUbtaRqQFEmGzbmpyZxhvO_t-1g2dsiiaghv3mC8e-7cc5dVr3Q8xetn0iAPT0fXsIFmxrC8sRqLWwgV4wVXz8p5MF4nXnBEuJ5w933QPe3OBQZdugPdYB0NKmVWxdwFep4Hyo1gSRT_6mJw_9O4G4SmIHQv-w-IzJIwRRHDCrtMnZzubsgR2-y-5HLGzV2JIigsdV57A3rBGYoGvrHRS8xLoLxwOYvPUWsvXcfH3RUKo3VTRS9rh3qYXbhmm414fnJHmhq4LllM4RoGPBubOrGZXS7HaynGpDGYoYAA3LaA0ILl6NXHnCGwKJVbWwAjEbq51RCifCeaiU8C6lzSGaMj-QmgXF2bP7uL1vzHfL2oCENmkyZqfLDjMESoW5QJeUy-oSmsP02KhzCrDbMaSBt4S41TcyfzgqZqO7SRJ8vYa8Q-RvQ0v8MGi6SgkW95kEoRL624BmC85LyPbPZpaKFIGx9Vui5HpftuYFQue4cBa4uEH3PwJfnQNA2upU8IinuMeodlQ6BNkEv4iAAXVm_jGhwLjS9y_6TQKEI_gYoVbyWaBXNU99VolVjzlfSg=".to_string();
/// let salt = b"P80Vnvk50aaQjyoJW8h65F70Q6IkLh0Newv2f2SAZlQ";
/// assert_eq!(hashed, hash(master, salt).unwrap())
/// ```
pub fn hash(string: &[u8], salt: &[u8]) -> Result<String, argon2::Error> {
    let mut hash_output = [0u8; 2048];
    let argon = Argon2::new(
        Algorithm::Argon2id,
        argon2::Version::V0x10,
        argon2::Params::new(2_u32.pow(16), 1, 3, Some(2048))?,
    );
    argon.hash_password_into(string, salt, &mut hash_output)?;
    Ok(URL_SAFE.encode(hash_output))
}

/// Checks if `hash_check` is equal to the hash of string (with salt)
/// # Example
/// ```
/// use nspm::cryptography::check_hash;
/// let master = "paper";
/// let hashed = "c4-u-v5GHsNjDwjaQkHkuIr3I60NqGz617NDQcV_bMH4ux7lScqyFQAEOBTSh9rX-82iGnXI3KLORz4TXVPn1ShqCUIIGBD53kbydHHwg_b5JALSopbICmPwjp2XkZ-lF3NbUD2h8hPIaeHOp-iSje9qHH4IiQ8DvYdK3UYTNhh8qQYpm6KMMKHXr3a3XnnOz80tcG9Y0BxlSfusS992CfMiEj_d2yZNlUUMZrgLWyIwtF1sUbZo7jzvI9qaol2cNxElCFmEcEKjW5joElUFYdpR02QOp-cbLZVQU5V_BLghrmj73usCKoTv7FUu4InKBhsnjtKnU2q3Vlvqyvj0Ekf5wITvgEnj6U4RL6ZRPsr3aV1pvzasvEIfhYy7R3VihRH2qqUAttPEnEb9TK2KMHEEZfuLk7U1rrmdijl_jC97IgDDMcunib9fAeKpxsA1rfQALaAXswHwmPjLU-UJqfiJ3NiVHSFKyMeNJH-KdoqnNo9kfGsSocnp5gJtXgLld-OQzstBgcUBUxWH7k6EMJCuGe9O6OcDMgncwOJXqxAx15wloOn5l7aq6Db5bHEcznYpAPKf1c19AvuTv7euIDJ4YUxZfhD6LNPbdLvXHc05Yl8_bXPAsW3L-YogAGdpKTeLpD2jNJ-rwIe9OWh5jnsXVY6w-utUEddlv0ZuHtJeUBJKKHU90Dhlu7bOqLrQbEGt-YTfE_zx5lDSym7R2_yGRQp85NfkHYVvkLn6i2j7Fc3RVggT1mYPt0hDi3Nbz5wPDoe6m-kwnchRaap6ZVGfn6P8gpr5ky7Q43Thl26J_reO20TMLpbysLE1ZFZV4O-ONw76F5NUdQYJ26YAt026voua2OjBom1UCTtvkvhP04bOFWAQiFmBfOFYTkBZkus0eRZTFvx3Y6LcTHLtn4K45cEC7VUOkhdMHlgwrEkHLzFMqDPOXBe9f9GUaeY8l4jqBH0ybadsxrv_UIMjmsBf7TcpidMF6TgKVmb79sjEldAlwGb2Hnq9s2BhSrwdrqoTIHS7SzH6l5pk002X6XOUQ2Ct0NlJ4P6__Xy_MmDLbo3o_bDulIxoHppxBAfRFtAg3tUhJ8DTXFdTkmxPtzcy1YNQtdWgsTzEpB0rwc51GlxMN_1oQB2ooADpa7AHC-r-s5cFBzXW5cIlqcpj7lBgPr0xVYwmj9tWnjJ4GjiwKTDeCguCUPr5FPSJMpwmdRCrhIufGpc2iJBbpfbR1H4CEvkYfP_ZYPKLQgNgq1WEwZnhGHjx4IgPgfzIl8_JOlu0hb5M3p8W8yXL_n47SGD3RMqO5a9TDEquX7JbpZfp6oWbvgmG6Ki5tneGu49FXNsP6--diYmRJ4CNveCBmNXkAuYLHEIgTWCx1ol2dJJ-iej4vamgpekDx-Y5P06tnGM-FlaJtFx6VtmNqrFFsrAGv5i43s5stKzANlP9wM44JznZgDq_w2TOrmY2BFlKHHMqFizIrSRZNm0S6vsDEBowQfIXG7XcKe-CBJaOWK1K5eYjfoR4gM1Ifr9tx8w9nGM3vq5MGcRGa_dwwOl_ypNErgcc9f8D-9HBVeF3synbno_OTq3phmE2V4Qvtmmadikdoxv7dP5wsb9skvSNOI41ngkFjsRdUFU0-DKm6veddJjSqX51VO7VdMfODmjOc5lusNeelKhULB21a9sPKzocE18YiiF3GhHUKvbLbCXBcWY25w9lGSNXBcD6s-inL_gQ_pRLPON5ctXi8kEyaqcwsMucaf014fNP_aPBoWL20J7Uomzt2Pv-0NlPmoYsg-_a1paWwmR1SU4rQ_Jre4SFaK4XkkeveI2vUvdjS2aXrkkCHJ8Pk00D4HZ0aY7tgZM0Ln9_EibV9gbkKZQRW3OzocL1GDuM3pUrBkzJWC-_n4MzER8npMQKyZ3Obbit8vJTPX-bnbw8hJm83ufo1DM9j0llTAkTcy9jg7eOrvpPv-xqDRYt_sOULwwEAvYPjrGJv1cPkGBFYo8FPYbk701INJTw1loewmjqUX0W87iYIl1eAvcqg_mwlgZ188sU9urIcgbw8ZR3DX5328YH3IBtEe7KDThaE4qNuGtN_YFUf6TS9j3IOQ5OLG06yrn0PM63T2XSl8GzE_iFiC9xqZQq3CWDWmUf5AAOQuW9zdfCUbtaRqQFEmGzbmpyZxhvO_t-1g2dsiiaghv3mC8e-7cc5dVr3Q8xetn0iAPT0fXsIFmxrC8sRqLWwgV4wVXz8p5MF4nXnBEuJ5w933QPe3OBQZdugPdYB0NKmVWxdwFep4Hyo1gSRT_6mJw_9O4G4SmIHQv-w-IzJIwRRHDCrtMnZzubsgR2-y-5HLGzV2JIigsdV57A3rBGYoGvrHRS8xLoLxwOYvPUWsvXcfH3RUKo3VTRS9rh3qYXbhmm414fnJHmhq4LllM4RoGPBubOrGZXS7HaynGpDGYoYAA3LaA0ILl6NXHnCGwKJVbWwAjEbq51RCifCeaiU8C6lzSGaMj-QmgXF2bP7uL1vzHfL2oCENmkyZqfLDjMESoW5QJeUy-oSmsP02KhzCrDbMaSBt4S41TcyfzgqZqO7SRJ8vYa8Q-RvQ0v8MGi6SgkW95kEoRL624BmC85LyPbPZpaKFIGx9Vui5HpftuYFQue4cBa4uEH3PwJfnQNA2upU8IinuMeodlQ6BNkEv4iAAXVm_jGhwLjS9y_6TQKEI_gYoVbyWaBXNU99VolVjzlfSg=".to_string();
/// let salt = "P80Vnvk50aaQjyoJW8h65F70Q6IkLh0Newv2f2SAZlQ";
/// assert!(check_hash(master, salt, &hashed))
/// ```
pub fn check_hash(string: &str, salt: &str, hash_check: &String) -> bool {
    hash(string.as_bytes(), salt.as_bytes()).unwrap() == *hash_check
}

/// Generates a salt of length 32 (trust me i need this)
pub fn generate_salt(
    rand: &mut impl rand_core::TryCryptoRng,
) -> Result<SaltString, argon2::password_hash::Error> {
    let mut buffer = [0u8; SALT_LENGTH];
    rand.try_fill_bytes(&mut buffer).unwrap();
    SaltString::encode_b64(&buffer)
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
        argon2::Version::V0x10,
        argon2::Params::new(2_u32.pow(16), 2, 3, Some(KEY_LENGTH)).unwrap(),
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
/// use argon2::password_hash::SaltString;
/// let salt = SaltString::from_b64("/NQctu0+XVTdWle/+JlMdT2lE+wIxELEHqIBebsypek").unwrap();
/// let master = b"p";
/// let fernet_encrypted = b"gAAAAABnon0g0-WEAZXvNmCzZg4NWXeY3aal7YcZh6yox6gppSCf8JYi_wKcmD5neL6sAFCc6oAUDJy3AomyilGpofezz3vH6g==";
/// assert_eq!(decrypt(fernet_encrypted, master, salt), String::from("p"))
/// ```
///
/// # Panics
///
/// Panics if master_pwd is not correct.
pub fn decrypt<'a>(pwd: &[u8], master_pwd: &[u8], salt: SaltString) -> Result<String, &'a str> {
    let mut key = [0u8; KEY_LENGTH];
    let buffer = pwd;
    let argon = Argon2::new_with_secret(
        master_pwd,
        Algorithm::Argon2id,
        argon2::Version::V0x10,
        argon2::Params::new(2_u32.pow(16), 2, 3, Some(KEY_LENGTH)).unwrap(),
    )
    .unwrap();
    let _ = argon.hash_password_into(master_pwd, salt.as_str().as_bytes(), &mut key);
    let key_b64 = URL_SAFE.encode(key);
    let f = Fernet::new(key_b64.as_str()).unwrap();
    let buffer_str = std::str::from_utf8(buffer).unwrap();
    let decrypted = f
        .decrypt(buffer_str)
        .map_err(|_e| "Error with decryption")?;
    let decrypted_str = String::from_utf8(decrypted).unwrap();
    Ok(decrypted_str)
}
