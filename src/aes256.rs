extern crate crypto;
extern crate rand;
use crate::crypto::mac::Mac;
use crate::rand::RngCore;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use console::style;
use crypto::buffer::{BufferResult, ReadBuffer, WriteBuffer};
use crypto::hmac::Hmac;
use crypto::sha2::Sha256;
use crypto::{aes, blockmodes, buffer, pbkdf2, symmetriccipher};
use serde::{Deserialize, Serialize};
use shellexpand;
use std::borrow::Borrow;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Write};

const KEY_CYCLES: u32 = 1000;
const SALT_CYCLES: u32 = 1000;
const IV_CYCLES: u32 = 1000;
const KEY_SIZE: usize = 256;
const IV_SIZE: usize = 16;
const BUF_SIZE: usize = 4096;
// let mut iv_copy = vec![0; IV_SIZE];
// iv_copy.copy_from_slice(&iv);
// let mut key_copy = vec![0; KEY_SIZE];
// key_copy.copy_from_slice(&key);

pub fn read_bytes(filename: &str) -> Vec<u8> {
    let f = File::open(filename).expect("failed to open file");
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();
    reader
        .read_to_end(&mut buffer)
        .expect("failed to read file");
    buffer
}

pub fn hmac_digest(mac_key: &[u8], iv: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::new(Sha256::new(), &mac_key);
    mac.input(&iv);
    let result = mac.result();
    let mac_digest = result.code();
    mac_digest.to_vec()
}
pub fn b64encode(bytes: &[u8]) -> String {
    let string = base64::encode(bytes);
    string
}

pub fn b64decode(bytes: &[u8]) -> Vec<u8> {
    let bytes = base64::decode(&bytes).unwrap();
    bytes
}

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
            iv: vec[3],
        }
    }
}
#[derive(PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub cycles: CyclesConfig,
}

impl Config {
    pub fn from_yaml(data: String) -> Config {
        let key: Config = serde_yaml::from_str(&data).expect("failed to deserialized yaml key");
        key
    }
    pub fn builtin() -> Config {
        Config {
            cycles: CyclesConfig {
                key: KEY_CYCLES,
                salt: SALT_CYCLES,
                iv: IV_CYCLES,
            },
        }
    }

    pub fn to_yaml(&self) -> String {
        match serde_yaml::to_string(&self) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("failed to encode key to yaml: {}", e);
                String::new()
            }
        }
    }

    pub fn default() -> Option<Config> {
        let filename = shellexpand::tilde("~/.toolz.yaml");
        Config::import(filename.borrow())
    }

    pub fn import(filename: &str) -> Option<Config> {
        match fs::read_to_string(filename) {
            Ok(yaml) => Some(Config::from_yaml(yaml)),
            Err(_) => Some(Config::builtin()),
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
#[derive(PartialEq, Serialize, Deserialize)]
pub struct AESKey {
    algo: String,
    key: String,
    mac: String,
    iv: String,
    magic: Option<Vec<u32>>,
}
impl AESKey {
    pub fn from_yaml(data: String) -> AESKey {
        let key: AESKey = serde_yaml::from_str(&data).expect("failed to deserialized yaml key");
        key
    }
    pub fn from_password(password: &[u8], config: &Config) -> AESKey {
        let iv = config.derive_iv(password);
        let salt = config.derive_salt(password);
        //let salt = generate_iv();
        let key_material = config.derive_key(password, &salt);

        let enc_key = &key_material[0..127];
        let mac_key = &key_material[128..255];

        AESKey {
            key: b64encode(&enc_key),
            mac: b64encode(&mac_key),
            iv: b64encode(&iv),
            algo: String::from("aes-256"),
            magic: Some(config.cycles.to_vec()),
        }
    }
    pub fn import(filename: &str) -> AESKey {
        let yaml = fs::read_to_string(filename).expect("cannot read key file");
        AESKey::from_yaml(yaml)
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
    pub fn to_yaml(&self) -> String {
        match serde_yaml::to_string(&self) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("failed to encode key to yaml: {}", e);
                String::new()
            }
        }
    }
    pub fn export(&self, filename: &str) -> String {
        let yaml = self.to_yaml();
        let mut file = File::create(filename).expect("failed to create new file");
        file.write(yaml.as_ref()).unwrap();
        String::from(filename)
    }

    // Encrypt a buffer with the given key and iv using
    // AES-256/CBC/Pkcs encryption.
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
        // Create an encryptor instance of the best performing
        // type available for the platform.
        let enc_key = self.key_bytes();
        //let mac_key = self.mac_bytes();
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

    // Decrypts a buffer with the given key and iv using
    // AES-256/CBC/Pkcs encryption.
    //
    // This function is very similar to encrypt(), so, please reference
    // comments in that function. In non-example code, if desired, it is possible to
    // share much of the implementation using closures to hide the operation
    // being performed. However, such code would make this example less clear.
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
pub fn confirm_password() -> Option<String> {
    let password = rpassword::prompt_password_stderr("Password: ").unwrap();
    let confirmation = rpassword::prompt_password_stderr("Confirm password: ").unwrap();

    if password != confirmation {
        eprintln!("{}", style("Password/Confirmation mismatch").color256(202));
        None
    } else {
        Some(password)
    }
}
fn get_password_from_matches(matches: &ArgMatches) -> String {
    let ask_password = matches.is_present("ask_password");
    let password = if ask_password {
        match confirm_password() {
            Some(password) => password,
            None => String::from(matches.value_of("password").unwrap_or("")),
        }
    } else {
        String::from(matches.value_of("password").unwrap_or(""))
    };
    password
}

fn load_key(matches: &ArgMatches, config: &Config) -> AESKey {
    let password = get_password_from_matches(matches);
    let key_filename = matches.value_of("key_filename").unwrap_or("");

    if key_filename.len() > 0 {
        AESKey::import(key_filename)
    } else if password.len() > 0 {
        AESKey::from_password(&password.as_bytes(), config)
    } else {
        panic!(
            "{}{}{}{}{}",
            style("either").color256(195),
            style("--password, --key-filename").color256(49),
            style(" or ").color256(195),
            style("--ask-password").color256(49),
            style(" is required").color256(195),
        );
    }
}

fn generate_command(matches: &ArgMatches, config: &Config) {
    let ask_password = matches.is_present("ask_password");
    let password = if ask_password {
        match confirm_password() {
            Some(password) => password,
            None => String::from(matches.value_of("password").unwrap_or("")),
        }
    } else {
        String::from(matches.value_of("password").unwrap_or(""))
    };
    let key = AESKey::from_password(&password.as_bytes(), config);

    let filename = matches.value_of("key_filename").unwrap();
    //let key_yaml = key.to_yaml();
    let key_path = key.export(filename);
    eprintln!(
        "{}{}",
        style("generated key: ").color256(44),
        style(key_path).color256(45)
    );
}
fn encrypt_command(matches: &ArgMatches, config: &Config) {
    let key = load_key(matches, config);
    let cyphertext_filename = matches.value_of("cyphertext_filename").unwrap();
    let plaintext_string = matches.value_of("string").unwrap_or("");
    let plaintext_filename = matches.value_of("plaintext_filename").unwrap_or("");

    let plaintext = if plaintext_filename.len() > 0 {
        read_bytes(plaintext_filename)
    } else if plaintext_string.len() > 0 {
        plaintext_string.as_bytes().to_vec()
    } else {
        panic!(
            "{}{}{}{}{}",
            style("either").color256(195),
            style("--string").color256(49),
            style(" or ").color256(195),
            style("--input-filename").color256(49),
            style(" is required").color256(195),
        );
    };

    let cyphertext = key.encrypt(&plaintext).ok().expect("encryption failed");
    let mut file = File::create(cyphertext_filename).expect("failed to create new file");
    file.write(&cyphertext).unwrap();
    eprintln!(
        "{}{}",
        style("wrote encrypted data in: ").color256(207),
        style(cyphertext_filename).color256(205)
    );
}

fn decrypt_command(matches: &ArgMatches, config: &Config) {
    let key_filename = matches.value_of("key_filename").unwrap_or("");

    let key = load_key(matches, config);

    let cyphertext_filename = matches.value_of("cyphertext_filename").unwrap();
    let plaintext_filename = matches.value_of("plaintext_filename").unwrap_or("");
    let cyphertext = read_bytes(cyphertext_filename);

    match key.decrypt(&cyphertext).ok() {
        Some(decrypted_data) => {
            if plaintext_filename.len() > 0 {
                let mut file = File::create(plaintext_filename).expect("failed to create new file");
                file.write(&decrypted_data)
                    .expect("failed to write to output filename");
                eprintln!(
                    "{}{}",
                    style("wrote plaintext data in: ").color256(49),
                    style(plaintext_filename).color256(45)
                );
            } else {
                println!("{}", b64encode(&decrypted_data));
            }
        }
        None => {
            eprintln!(
                "{}",
                style(format!(
                    "failed to decrypt {} {} {}",
                    style(cyphertext_filename).color256(49),
                    style("with key").color256(202),
                    style(key_filename).color256(45),
                ))
                .color256(202)
            );
            std::process::exit(1);
        }
    }
}

fn main() {
    let app = App::new("aes256")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version("1.0")
        .author("Gabriel Falcão <gabriel@nacaolivre.org>")
        .about("perform aes-256-cbc encryption/decryption based on PBKDF2 of password")
        .arg(
            Arg::with_name("dry_run")
                .long("dry-run")
                .short("n")
                .takes_value(false),
        )
        // .subcommand(
        //     SubCommand::with_name("ask")
        //         .about("ask for password and confirmation")
        //         .arg(
        //             Arg::with_name("force")
        //                 .long("force")
        //                 .short("f")
        //                 .takes_value(false),
        //         ),
        // )
        .subcommand(
            SubCommand::with_name("generate")
                .about("generate key")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .short("k")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("password")
                        .long("password")
                        .short("P")
                        .required_unless_one(&["key_filename", "ask_password"])
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ask_password")
                        .long("ask-password")
                        .short("p")
                        .required_unless_one(&["password", "ask_password"])
                        .takes_value(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("encrypt")
                .about("encrypt file or string")
                .arg(
                    Arg::with_name("string")
                        .long("string")
                        .short("s")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("plaintext_filename")
                        .long("input-filename")
                        .short("i")
                        .required_unless("string")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("cyphertext_filename")
                        .long("output-filename")
                        .short("o")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("password")
                        .long("password")
                        .short("P")
                        .required_unless_one(&["key_filename", "ask_password"])
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ask_password")
                        .long("ask-password")
                        .required(false)
                        .short("p")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .short("k")
                        .required_unless_one(&["password", "ask_password"])
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("decrypt")
                .about("decrypt file")
                .arg(
                    Arg::with_name("password")
                        .long("password")
                        .short("P")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .short("k")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ask_password")
                        .long("ask-password")
                        .short("p")
                        .required(false)
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("plaintext_filename")
                        .long("output-filename")
                        .short("o")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("cyphertext_filename")
                        .long("input-filename")
                        .short("i")
                        .required(true)
                        .takes_value(true),
                ),
        );

    let matches = app.get_matches();
    //let dry_run = matches.is_present("dry_run");

    let config = Config::default().expect("cannot read default config: ~/.toolz.yaml");

    match matches.subcommand() {
        ("generate", Some(matches)) => {
            generate_command(matches, &config);
        }
        ("encrypt", Some(matches)) => {
            encrypt_command(matches, &config);
        }
        ("decrypt", Some(matches)) => {
            decrypt_command(matches, &config);
        }
        (cmd, Some(_matches)) => {
            eprintln!("command not implemented: {}", cmd);
        }
        (cmd, None) => {
            eprintln!("unhandled command: {}", cmd);
        }
    }
}
