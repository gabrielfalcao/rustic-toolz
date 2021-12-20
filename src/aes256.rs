use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use console::style;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use toolz::aes256cbc::b64encode;

use toolz::aes256cbc::Config;
use toolz::aes256cbc::Key;
use toolz::core;

pub fn read_bytes(filename: &str) -> Vec<u8> {
    let f = File::open(filename).expect("failed to open file");
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();
    reader
        .read_to_end(&mut buffer)
        .expect("failed to read file");
    buffer
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

fn load_key(matches: &ArgMatches, config: &Config) -> Key {
    let password = get_password_from_matches(matches);
    let key_filename = matches.value_of("key_filename").unwrap_or("");

    if key_filename.len() > 0 {
        Key::import(key_filename)
    } else if password.len() > 0 {
        Key::from_password(&password.as_bytes(), config)
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
    // test
    // C-x C-s to save, just like in emacs

    let password = if ask_password {
        match confirm_password() {
            Some(password) => password,
            None => String::from(matches.value_of("password").unwrap_or("")),
        }
    } else {
        String::from(matches.value_of("password").unwrap_or(""))
    };
    let key = Key::from_password(&password.as_bytes(), config);

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

    if key.owns_file(plaintext_filename) {
        eprintln!(
            "{}{}",
            style("skipping file already encrypted: ").color256(162),
            style(plaintext_filename).color256(136)
        );
        std::process::exit(1);
    }

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
    println!(
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

    if !key.owns_file(cyphertext_filename) {
        eprintln!(
            "{}{}",
            style("skipping file not owned by the given key: ").color256(203),
            style(cyphertext_filename).color256(208)
        );
        return;
    }

    let cyphertext = read_bytes(cyphertext_filename);

    match key.decrypt(&cyphertext).ok() {
        Some(decrypted_data) => {
            if plaintext_filename.len() > 0 {
                let mut file = File::create(plaintext_filename).expect("failed to create new file");
                file.write(&decrypted_data)
                    .expect("failed to write to output filename");
                println!(
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
        .version(core::VERSION)
        .author(core::AUTHOR)
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
