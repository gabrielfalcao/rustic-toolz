extern crate clap;
use clap::{App, Arg};
use console::style;

use bip39::{Language, Mnemonic, MnemonicType, Seed};

fn main() {
    let app = App::new("bip39").about("generate a bip39 key").arg(
        Arg::with_name("type")
            .help("mnemonic type")
            .long("type")
            .short("t")
            .value_name("<Words12 | Words15 | Words18 | Words21 | Words24>")
            .default_value("Words24")
            .takes_value(true),
    );

    let matches = app.get_matches();
    let mtype = matches.value_of("type").unwrap();

    let mnemonic = Mnemonic::new(
        match mtype {
            "Words12" => MnemonicType::Words12,
            "Words15" => MnemonicType::Words15,
            "Words18" => MnemonicType::Words18,
            "Words21" => MnemonicType::Words21,
            "Words24" => MnemonicType::Words24,
            invalid => {
                panic!("Invalid mnemonic type: {}", invalid);
            }
        },
        Language::English,
    );

    let phrase: &str = mnemonic.phrase();
    println!("phrase: {}", style(phrase).color256(207));

    let seed = Seed::new(&mnemonic, "");

    // // get the HD wallet seed as raw bytes
    // let seed_bytes: &[u8] = seed.as_bytes();

    // print the HD wallet seed as a hex string
    println!("{:X}", seed);
}
