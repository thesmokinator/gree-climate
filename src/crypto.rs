use aes::Aes128;
use aes::cipher::{BlockEncrypt, BlockDecrypt, KeyInit};
use aes::cipher::generic_array::GenericArray;
use base64::Engine;
use tracing::debug;

use crate::error::Error;
use crate::utils::{pkcs7_pad, pkcs7_unpad, reset_trailing_garbage};

const V1_DEFAULT_KEY: &str = "a3K8Bx%2r8Y7#xDh";
const V2_DEFAULT_KEY: &str = "{yxAHAY_Lm6pbC/<";
const V2_NONCE: [u8; 12] = [0x54, 0x40, 0x78, 0x44, 0x49, 0x67, 0x5a, 0x51, 0x6c, 0x5e, 0x63, 0x13];

use aes_gcm::{
    aead::Aead,
    Aes128Gcm,
};

#[derive(Debug, Clone)]
pub enum Cipher {
    V1(CipherV1),
    V2(CipherV2),
}

impl Cipher {
    pub fn v1() -> Self {
        Self::V1(CipherV1::new())
    }

    pub fn v2() -> Self {
        Self::V2(CipherV2::new())
    }

    pub fn v1_with_key(key: &str) -> Self {
        Self::V1(CipherV1::with_key(key))
    }

    pub fn v2_with_key(key: &str) -> Self {
        Self::V2(CipherV2::with_key(key))
    }

    pub fn encrypt(&self, data: &str) -> Result<(String, Option<String>), Error> {
        match self {
            Self::V1(c) => c.encrypt(data),
            Self::V2(c) => c.encrypt(data),
        }
    }

    pub fn decrypt(&self, data: &str) -> Result<String, Error> {
        match self {
            Self::V1(c) => c.decrypt(data),
            Self::V2(c) => c.decrypt(data),
        }
    }

    pub fn set_key(&mut self, key: &str) {
        match self {
            Self::V1(c) => c.set_key(key),
            Self::V2(c) => c.set_key(key),
        }
    }

    pub fn key(&self) -> &str {
        match self {
            Self::V1(c) => c.key(),
            Self::V2(c) => c.key(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CipherV1 {
    key: Vec<u8>,
}

impl CipherV1 {
    pub fn new() -> Self {
        Self {
            key: V1_DEFAULT_KEY.as_bytes().to_vec(),
        }
    }

    pub fn with_key(key: &str) -> Self {
        Self {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn set_key(&mut self, key: &str) {
        self.key = key.as_bytes().to_vec();
    }

    pub fn key(&self) -> &str {
        std::str::from_utf8(&self.key).unwrap_or_default()
    }

    fn create_cipher(&self) -> Aes128 {
        let key = GenericArray::from_slice(&self.key);
        Aes128::new(key)
    }

    pub fn encrypt(&self, data: &str) -> Result<(String, Option<String>), Error> {
        debug!("CipherV1 encrypting: {data}");
        let cipher = self.create_cipher();
        let padded = pkcs7_pad(data.as_bytes(), 16);

        let mut encrypted = Vec::with_capacity(padded.len());
        for chunk in padded.chunks(16) {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.encrypt_block(&mut block);
            encrypted.extend_from_slice(&block);
        }

        let encoded = base64::engine::general_purpose::STANDARD.encode(&encrypted);
        debug!("CipherV1 encrypted: {encoded}");
        Ok((encoded, None))
    }

    pub fn decrypt(&self, data: &str) -> Result<String, Error> {
        debug!("CipherV1 decrypting: {data}");
        let cipher = self.create_cipher();
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|e| Error::Crypto(format!("base64 decode failed: {e}")))?;

        if decoded.len() % 16 != 0 {
            return Err(Error::Crypto(format!(
                "ciphertext length {} is not a multiple of 16",
                decoded.len()
            )));
        }

        let mut decrypted = Vec::with_capacity(decoded.len());
        for chunk in decoded.chunks(16) {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.decrypt_block(&mut block); // ECB encrypt = decrypt for block cipher? No, decrypt is decrypt.
            // Actually, AES decrypt_block should be used for decryption.
            // But we're decrypting ECB, so each block is decrypted independently.
            decrypted.extend_from_slice(&block);
        }

        let plain = pkcs7_unpad(&decrypted)?;
        let text = String::from_utf8(plain.to_vec())
            .map_err(|e| Error::Crypto(format!("utf8 decode failed: {e}")))?;
        let clean = reset_trailing_garbage(&text);
        debug!("CipherV1 decrypted: {clean}");
        Ok(clean)
    }
}

#[derive(Debug, Clone)]
pub struct CipherV2 {
    key: Vec<u8>,
}

impl CipherV2 {
    pub fn new() -> Self {
        Self {
            key: V2_DEFAULT_KEY.as_bytes().to_vec(),
        }
    }

    pub fn with_key(key: &str) -> Self {
        Self {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn set_key(&mut self, key: &str) {
        self.key = key.as_bytes().to_vec();
    }

    pub fn key(&self) -> &str {
        std::str::from_utf8(&self.key).unwrap_or_default()
    }

    pub fn encrypt(&self, data: &str) -> Result<(String, Option<String>), Error> {
        debug!("CipherV2 encrypting: {data}");
        let key = GenericArray::clone_from_slice(&self.key);
        let cipher = Aes128Gcm::new(&key);
        let encrypted = cipher
            .encrypt(
                GenericArray::from_slice(&V2_NONCE),
                data.as_bytes(),
            )
            .map_err(|e| Error::Crypto(format!("encryption failed: {e}")))?;

        let encoded = base64::engine::general_purpose::STANDARD.encode(&encrypted);
        debug!("CipherV2 encrypted: {encoded}");
        Ok((encoded, None))
    }

    pub fn decrypt(&self, data: &str) -> Result<String, Error> {
        debug!("CipherV2 decrypting: {data}");
        let key = GenericArray::clone_from_slice(&self.key);
        let cipher = Aes128Gcm::new(&key);
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|e| Error::Crypto(format!("base64 decode failed: {e}")))?;

        let plain = cipher
            .decrypt(GenericArray::from_slice(&V2_NONCE), decoded.as_ref())
            .map_err(|e| Error::Crypto(format!("decryption failed: {e}")))?;

        let text = String::from_utf8(plain)
            .map_err(|e| Error::Crypto(format!("utf8 decode failed: {e}")))?;
        let clean = reset_trailing_garbage(&text);
        debug!("CipherV2 decrypted: {clean}");
        Ok(clean)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cipher_v1_encrypt_decrypt() {
        let cipher = CipherV1::new();
        let data = r#"{"t":"bind","mac":"aabbcc112233","uid":0}"#;
        let (encrypted, tag) = cipher.encrypt(data).unwrap();
        assert!(tag.is_none());
        assert!(!encrypted.is_empty());
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_cipher_v1_with_simple_data() {
        let cipher = CipherV1::new();
        let data = r#"hello world this is a test"#;
        let (encrypted, _) = cipher.encrypt(data).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_cipher_v2_encrypt_decrypt() {
        let cipher = CipherV2::new();
        let data = r#"{"t":"status","cols":["Pow","Mod"]}"#;
        let (encrypted, tag) = cipher.encrypt(data).unwrap();
        assert!(tag.is_none());
        assert!(!encrypted.is_empty());
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_cipher_v2_with_key() {
        let mut cipher = CipherV2::with_key("my-custom-keys!");
        cipher.set_key("abcdefghijklmnop");
        let data = r#"{"test": true}"#;
        let (encrypted, _) = cipher.encrypt(data).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_cipher_enum_v1() {
        let cipher = Cipher::v1();
        let data = r#"{"t":"bind","mac":"test"}"#;
        let (encrypted, _) = cipher.encrypt(data).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_cipher_enum_v2() {
        let cipher = Cipher::v2();
        let data = r#"{"t":"scan"}"#;
        let (encrypted, _) = cipher.encrypt(data).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_v1_default_key_matches_reference() {
        let cipher = CipherV1::new();
        assert_eq!(cipher.key(), V1_DEFAULT_KEY);
    }

    #[test]
    fn test_v2_default_key_matches_reference() {
        let cipher = CipherV2::new();
        assert_eq!(cipher.key(), V2_DEFAULT_KEY);
    }
}
