use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use keyring::Entry;
use rand::RngCore;

#[allow(dead_code)]
const SERVICE_NAME: &str = "valo-accounts";
#[allow(dead_code)]
const KEY_NAME: &str = "encryption_key";

#[allow(dead_code)]
pub fn get_or_create_encryption_key() -> Result<Vec<u8>, String> {
    let entry = Entry::new(SERVICE_NAME, KEY_NAME)
        .map_err(|e| format!("Failed to access Windows Credential Manager: {}", e))?;

    match entry.get_password() {
        Ok(key_str) => {
            general_purpose::STANDARD
                .decode(&key_str)
                .map_err(|e| format!("Failed to decode encryption key: {}", e))
        }
        Err(_) => {
            let new_key = generate_random_key();
            let key_str = general_purpose::STANDARD.encode(&new_key);
            entry
                .set_password(&key_str)
                .map_err(|e| format!("Failed to store encryption key: {}", e))?;
            Ok(new_key)
        }
    }
}

#[allow(dead_code)]
fn generate_random_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

#[allow(dead_code)]
pub fn encrypt_password(password: &str, key: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 32 {
        return Err("Encryption key must be 32 bytes".to_string());
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let mut ciphertext = cipher
        .encrypt(nonce, password.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut result = nonce_bytes.to_vec();
    result.append(&mut ciphertext);

    Ok(result)
}

#[allow(dead_code)]
pub fn decrypt_password(encrypted: &[u8], key: &[u8]) -> Result<String, String> {
    if key.len() != 32 {
        return Err("Encryption key must be 32 bytes".to_string());
    }

    if encrypted.len() < 12 {
        return Err("Invalid encrypted data".to_string());
    }

    let (nonce_bytes, ciphertext) = encrypted.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 conversion error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let password = "TestPassword123!";
        let key = generate_random_key();

        let encrypted = encrypt_password(password, &key).unwrap();
        let decrypted = decrypt_password(&encrypted, &key).unwrap();

        assert_eq!(password, decrypted);
    }

    #[test]
    fn test_different_nonces() {
        let password = "TestPassword123!";
        let key = generate_random_key();

        let encrypted1 = encrypt_password(password, &key).unwrap();
        let encrypted2 = encrypt_password(password, &key).unwrap();

        assert_ne!(encrypted1, encrypted2);

        let decrypted1 = decrypt_password(&encrypted1, &key).unwrap();
        let decrypted2 = decrypt_password(&encrypted2, &key).unwrap();

        assert_eq!(decrypted1, decrypted2);
        assert_eq!(password, decrypted1);
    }
}
