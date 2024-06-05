use clap::Parser;
use crate::cli::{CliCommands};
use std::char;
use std::collections::HashSet;
use std::io::Write;
use std::str;
use std::string::String;

mod cli;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = cli::Cli::parse();

    let device_identifier = "example_device";
    let device_key = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    match cli.command {
        CliCommands::Link { name } => {
            create_device_link_uri(device_identifier, &device_key);
        }
        CliCommands::Version => println!("Version")
    }
    Ok(())
}

fn create_device_link_uri(device_identifier: &str, device_key: &[u8]){
    let device_key_string = base64::encode(device_key).replace("=", "");
    let uri_string = format!(
        "sgnl://linkdevice?uuid={}&pub_key={}",
        encode(device_identifier),
        encode(&device_key_string)
    );
    println!("{}", uri_string);
    //uri_string.parse().expect("Failed to parse URI")
}

fn encode(s: &str) -> String {
    let charset = "UTF-8";
    let charset_set: HashSet<char> = charset.chars().collect();

    let dont_need_encoding: HashSet<char> = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
        'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1',
        '2', '3', '4', '5', '6', '7', '8', '9', '-', '_', '.', '~',
    ]
        .iter()
        .cloned()
        .collect();

    let mut out = String::with_capacity(s.len());
    let mut char_array_writer = Vec::new();

    let mut i = 0;
    while i < s.len() {
        let c = s.chars().nth(i).unwrap();
        if dont_need_encoding.contains(&c) {
            if c == ' ' {
                out.push('+');
            } else {
                out.push(c);
            }
            i += 1;
        } else {
            char_array_writer.push(c);
            i += 1;

            while i < s.len() && !dont_need_encoding.contains(&s.chars().nth(i).unwrap()) {
                char_array_writer.push(s.chars().nth(i).unwrap());
                i += 1;
            }

            let str_chars: String = char_array_writer.iter().collect();
            let str_bytes = str_chars.as_bytes();

            for b in str_bytes {
                out.push('%');
                let ch = char::from_digit((b >> 4) as u32 & 0xF, 16).unwrap();
                if charset_set.contains(&ch) {
                    out.push(ch.to_ascii_uppercase());
                } else {
                    out.push(ch);
                }
                let ch = char::from_digit(*b as u32 & 0xF, 16).unwrap();
                if charset_set.contains(&ch) {
                    out.push(ch.to_ascii_uppercase());
                } else {
                    out.push(ch);
                }
            }
            char_array_writer.clear();
        }
    }
    out
}

