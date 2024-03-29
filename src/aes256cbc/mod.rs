/*!
aes-256-cbc module

This library provides user-friendly utilities for performing AES-256-CBC operations.

Currenly supports:

- key derivation with password
- encryption
- decryption

# Example

This example shows how to create a "standard" printer and execute a search.

```
use toolz::aes256cbc::{Key, Config};

let config = Config::from_vec(&[100, 200, 300]);

let password = String::from("I <3 Nickelback");
let key = Key::from_password(&password.as_bytes(), &config);

let plaintext = b"Some secret information";
let cyphertext = key.encrypt(plaintext).ok().expect("encryption failed");

let decrypted = key.decrypt(&cyphertext).ok().expect("decryption failed");

assert_eq!((*plaintext).to_vec(), decrypted);
```
*/

extern crate crypto;
extern crate rand;

use crypto::buffer::{BufferResult, ReadBuffer, WriteBuffer};
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use crypto::{aes, blockmodes, buffer, pbkdf2, symmetriccipher};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use shellexpand;
use std::borrow::Borrow;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Write};

const ALGO: &'static str = "aes-256-cbc";
const DIGEST_SIZE: usize = 32;
///The path used by `Config::default()`
const DEFAULT_CONFIG_PATH: &'static str = "~/.rustic-toolz.yaml";

///The builtin number of cycles for a key derivation
const KEY_CYCLES: u32 = 1000;
///The builtin number of cycles for a salt derivation
const SALT_CYCLES: u32 = 1000;
///The builtin number of cycles for a ivv derivation
const IV_CYCLES: u32 = 1000;

const KEY_SIZE: usize = 128;
const IV_SIZE: usize = 16;
const BUF_SIZE: usize = 4096;

/// Reads the given filename as Vec<u8>
pub fn read_bytes(filename: &str) -> Vec<u8> {
    let f = File::open(filename).expect("failed to open file");
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();
    reader
        .read_to_end(&mut buffer)
        .expect("failed to read file");
    buffer
}
pub fn bytes_match(a: &[u8], b: &[u8]) -> bool {
    let diff = a
        .iter()
        .zip(b.iter())
        .map(|(a, b)| a ^ b)
        .fold(0, |acc, x| acc | x);
    diff == 0 && a.len() == b.len()
}
/// Dummy example of hmac_256_digest
pub fn hmac_256_digest(mac_key: &[u8], iv: &[u8]) -> [u8; DIGEST_SIZE] {
    let mut mac = Hmac::new(Sha256::new(), &mac_key);
    mac.input(&iv);
    let result = mac.result();
    let mac_digest = result.code();
    mac_digest[..DIGEST_SIZE].try_into().unwrap()
}

/// Encodes &[u8] to a base64 string
///
/// # Example
///
/// ```
/// use toolz::aes256cbc::b64encode;
/// assert_eq!("SGVsbG8=", b64encode(b"Hello"));
/// ```
pub fn b64encode(bytes: &[u8]) -> String {
    let string = base64::encode(bytes);
    string
}

/// Encodes base64 string into a Vec<8>
///
/// # Example
///
/// ```
/// use toolz::aes256cbc::b64decode;
/// assert_eq!(b"Hello".to_vec(), b64decode(b"SGVsbG8="));
/// ```
pub fn b64decode(bytes: &[u8]) -> Vec<u8> {
    let bytes = base64::decode(&bytes).unwrap();
    bytes
}

/// Generates a random KEY;
pub fn generate_key() -> [u8; KEY_SIZE] {
    let mut rng = rand::thread_rng();
    let mut key: [u8; KEY_SIZE] = [0; KEY_SIZE];
    rng.fill_bytes(&mut key);
    key
}
/// Generates a random IV;
pub fn generate_iv() -> [u8; IV_SIZE] {
    let mut rng = rand::thread_rng();
    let mut iv: [u8; IV_SIZE] = [0; IV_SIZE];
    rng.fill_bytes(&mut iv);
    iv
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct CyclesConfig {
    pub key: u32,
    pub salt: u32,
    pub iv: u32,
}
impl CyclesConfig {
    pub fn to_vec(&self) -> Vec<u32> {
        let mut cycles: Vec<u32> = Vec::new();
        cycles.push(self.key);
        cycles.push(self.salt);
        cycles.push(self.iv);
        cycles
    }
    pub fn from_vec(vec: &[u32; 3]) -> CyclesConfig {
        CyclesConfig {
            key: vec[0],
            salt: vec[1],
            iv: vec[2],
        }
    }
}

/// The configuration for the Key.
///
/// It contains the cycles for key, salt and iv used in key derivation.
#[derive(PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub cycles: CyclesConfig,
    pub default_key_path: Option<String>,
}

impl Config {
    /// Creates a new config based on a YAML-serialized string
    pub fn from_yaml(data: String) -> Config {
        let key: Config = serde_yaml::from_str(&data).expect("failed to deserialized yaml key");
        key
    }
    /// Creates a new config based on a &Vec<u32>
    pub fn from_vec(vec: &[u32; 3]) -> Config {
        Config {
            cycles: CyclesConfig::from_vec(vec),
            default_key_path: None,
        }
    }
    /// Creates a new builtin config
    pub fn builtin(default_key_path: Option<String>) -> Config {
        Config {
            default_key_path,
            cycles: CyclesConfig {
                key: KEY_CYCLES,
                salt: SALT_CYCLES,
                iv: IV_CYCLES,
            },
        }
    }

    /// Exports config to a yaml string
    pub fn to_yaml(&self) -> String {
        match serde_yaml::to_string(&self) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("failed to encode key to yaml: {}", e);
                String::new()
            }
        }
    }

    /// Loads the default config from `DEFAULT_CONFIG_PATH`
    pub fn default() -> Option<Config> {
        let filename = shellexpand::tilde(DEFAULT_CONFIG_PATH);
        Config::import(filename.borrow())
    }
    /// Loads the default config from a yaml file
    pub fn import(filename: &str) -> Option<Config> {
        match fs::read_to_string(filename) {
            Ok(yaml) => Some(Config::from_yaml(yaml)),
            Err(_) => Some(Config::builtin(None)),
        }
    }
    pub fn iv_cycles(&self) -> u32 {
        self.cycles.iv
    }
    pub fn key_cycles(&self) -> u32 {
        self.cycles.key
    }
    pub fn salt_cycles(&self) -> u32 {
        self.cycles.salt
    }
    pub fn derive_key<'a>(&self, password: &[u8], salt: &[u8]) -> [u8; KEY_SIZE] {
        let mut dk = [0u8; KEY_SIZE]; // derived key
        let mut mac = Hmac::new(Sha256::new(), password);
        pbkdf2::pbkdf2(&mut mac, &salt, self.key_cycles(), &mut dk);
        dk
    }
    pub fn derive_salt<'a>(&self, password: &[u8]) -> [u8; KEY_SIZE] {
        let mut dk = [0u8; KEY_SIZE]; // derived key
        let mut mac = Hmac::new(Sha256::new(), password);
        pbkdf2::pbkdf2(&mut mac, &password, self.salt_cycles(), &mut dk);
        dk
    }
    pub fn derive_iv<'a>(&self, password: &[u8]) -> [u8; IV_SIZE] {
        let mut dk = [0u8; IV_SIZE]; // derived iv
        let mut mac = Hmac::new(Sha256::new(), password);
        pbkdf2::pbkdf2(&mut mac, &password, self.iv_cycles(), &mut dk);
        dk
    }
}
/// AES-256 Key data
#[derive(PartialEq, Serialize, Deserialize)]
pub struct Key {
    pub algo: String,
    pub key: String,
    pub mac: String,
    pub iv: String,
    pub magic: Option<Vec<u32>>,
}
impl Key {
    /// Load a key from a yaml string
    pub fn from_yaml(data: String) -> Key {
        let key: Key = serde_yaml::from_str(&data).expect("failed to deserialized yaml key");
        key
    }
    /// Derive a key from a password using the cycles from the given config
    pub fn from_password(password: &[u8], config: &Config) -> Key {
        let iv = config.derive_iv(password);
        let salt = config.derive_salt(password);
        //let salt = generate_iv();
        let key_material = config.derive_key(password, &salt);

        let enc_key = &key_material[0..127];
        let mac_key = &key_material[128..255];

        Key {
            key: b64encode(&enc_key),
            mac: b64encode(&mac_key),
            iv: b64encode(&iv),
            algo: String::from(ALGO),
            magic: Some(config.cycles.to_vec()),
        }
    }
    /// Generate a new key
    pub fn generate() -> Key {
        let iv = generate_iv();
        let key_material = generate_key();
        let enc_key = &key_material[0..127];
        let mac_key = &key_material[128..255];

        Key {
            key: b64encode(&enc_key),
            mac: b64encode(&mac_key),
            iv: b64encode(&iv),
            algo: String::from(ALGO),
            magic: None,
        }
    }
    /// Checks if a file is encrypted with this key
    pub fn owns_file(&self, filename: &str) -> bool {
        let mut fd =
            File::open(filename).expect(format!("failed to open file {}", filename).as_str());
        let mut buffer = [0; DIGEST_SIZE];
        fd.read(&mut buffer)
            .expect(format!("failed to read the first bytes of {}", filename).as_str());

        self.check_digest(&buffer)
    }
    /// Checks the digest of the given bytes
    pub fn check_digest(&self, buffer: &[u8; DIGEST_SIZE]) -> bool {
        let digest = self.digest();
        bytes_match(buffer, &digest)
    }
    /// Load key from a YAML file
    pub fn import(filename: &str) -> Key {
        let yaml = fs::read_to_string(filename).expect("cannot read key file");
        Key::from_yaml(yaml)
    }
    pub fn digest(&self) -> [u8; DIGEST_SIZE] {
        let mac = self.mac_bytes();
        let iv = self.iv_bytes();
        hmac_256_digest(&mac, &iv)
    }
    pub fn iv_bytes(&self) -> Vec<u8> {
        b64decode(self.iv.as_bytes())
    }
    pub fn key_bytes(&self) -> Vec<u8> {
        b64decode(self.key.as_bytes())
    }
    pub fn mac_bytes(&self) -> Vec<u8> {
        b64decode(self.mac.as_bytes())
    }
    /// Serialize key into a YAML string
    pub fn to_yaml(&self) -> String {
        match serde_yaml::to_string(&self) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("failed to encode key to yaml: {}", e);
                String::new()
            }
        }
    }
    /// Store YAML-serialized key into a file
    pub fn export(&self, filename: &str) -> String {
        let yaml = self.to_yaml();
        let mut file = File::create(filename).expect("failed to create new file");
        file.write(yaml.as_ref()).unwrap();
        String::from(filename)
    }

    /// Encrypt a buffer with the key
    /// AES-256/CBC/Pkcs encryption.
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
        // Create an encryptor instance of the best performing
        // type available for the platform.
        let enc_key = self.key_bytes();
        let iv = self.iv_bytes();
        let mut encryptor = aes::cbc_encryptor(
            aes::KeySize::KeySize256,
            &enc_key,
            &iv,
            blockmodes::PkcsPadding,
        );

        // Each encryption operation encrypts some data from
        // an input buffer into an output buffer. Those buffers
        // must be instances of RefReaderBuffer and RefWriteBuffer
        // (respectively) which keep track of how much data has been
        // read from or written to them.
        let mut cyphertext = Vec::<u8>::new();
        let mut read_buffer = buffer::RefReadBuffer::new(data);
        let mut buffer = [0; BUF_SIZE];
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        // The first 32 bytes of the cyphertext are the digest
        let digest = self.digest();
        cyphertext.extend_from_slice(&digest);

        // Each encryption operation will "make progress". "Making progress"
        // is a bit loosely defined, but basically, at the end of each operation
        // either BufferUnderflow or BufferOverflow will be returned (unless
        // there was an error). If the return value is BufferUnderflow, it means
        // that the operation ended while wanting more input data. If the return
        // value is BufferOverflow, it means that the operation ended because it
        // needed more space to output data. As long as the next call to the encryption
        // operation provides the space that was requested (either more input data
        // or more output space), the operation is guaranteed to get closer to
        // completing the full operation - ie: "make progress".
        //
        // Here, we pass the data to encrypt to the enryptor along with a fixed-size
        // output buffer. The 'true' flag indicates that the end of the data that
        // is to be encrypted is included in the input buffer (which is true, since
        // the input data includes all the data to encrypt). After each call, we copy
        // any output data to our result Vec. If we get a BufferOverflow, we keep
        // going in the loop since it means that there is more work to do. We can
        // complete as soon as we get a BufferUnderflow since the encryptor is telling
        // us that it stopped processing data due to not having any more data in the
        // input buffer.
        loop {
            let result = encryptor.encrypt(&mut read_buffer, &mut write_buffer, true)?;

            // "write_buffer.take_read_buffer().take_remaining()" means:
            // from the writable buffer, create a new readable buffer which
            // contains all data that has been written, and then access all
            // of that data as a slice.
            cyphertext.extend(
                write_buffer
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .map(|&i| i),
            );

            match result {
                BufferResult::BufferUnderflow => break,
                BufferResult::BufferOverflow => {}
            }
        }
        Ok(cyphertext)
    }

    /// Decrypts a buffer with the key
    /// AES-256/CBC/Pkcs decryption.
    pub fn decrypt(
        &self,
        cyphertext: &[u8],
    ) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
        let mut decryptor = aes::cbc_decryptor(
            aes::KeySize::KeySize256,
            &self.key_bytes(),
            &self.iv_bytes(),
            blockmodes::PkcsPadding,
        );

        let mut plaintext = Vec::<u8>::new();
        let hmac_bytes: [u8; DIGEST_SIZE] = cyphertext[..DIGEST_SIZE].try_into().unwrap();
        if !self.check_digest(&hmac_bytes) {
            eprintln!("Cannot decrypt: data was not encrypted by this key");
            return Ok((*cyphertext).to_vec());
        }

        let cyphertext = &cyphertext[DIGEST_SIZE..];
        let mut read_buffer = buffer::RefReadBuffer::new(&cyphertext);
        let mut buffer = [0; BUF_SIZE];
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        loop {
            let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true)?;

            plaintext.extend(
                write_buffer
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .map(|&i| i),
            );
            match result {
                BufferResult::BufferUnderflow => break,
                BufferResult::BufferOverflow => {}
            }
        }

        Ok(plaintext)
    }
}
#[cfg(test)]
mod tests {
    use crate::aes256cbc::Config;
    use crate::aes256cbc::Key;
    use k9::assert_equal;

    #[test]
    fn test_encrypt_and_decrypt() {
        let config = Config::builtin(None);
        let password = String::from("123456");
        let key = Key::from_password(&password.as_bytes(), &config);

        let plaintext = b"This is a secret";
        let cyphertext = key.encrypt(plaintext).unwrap();

        let decrypted = key.decrypt(&cyphertext).unwrap();
        assert_equal!(decrypted, b"This is a secret");
    }
}
