extern crate core;
extern crate qrcodegen;

mod functions;
use std::env;
use crate::functions::accounts::{ link_account};
use crate::functions::contacts::sync_and_print_contacts;
use crate::functions::received::{receive_and_store_messages, show_messages};
use crate::functions::sending::send_message;

fn print_options(){
    println!("Please use one of the following options:");
    println!("  send <recipient> <message>   - Send a message to a recipient");
    println!("  account <account_name>      - Link an account");
    println!("  receive                     - Receive and store messages");
    println!("  show <contact>              - Show messages for a contact");
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("WELCOME TO OUR SIGNAL-CLIENT!\n");
        print_options();
        return Ok(());
    }

    let option = &args[1];
    match option.as_str() {
        "send" => send_message(args).await?,
        "account" => link_account(args).await?,
        "receive" => receive_and_store_messages().await?,
        "show" => show_messages(args).await?,
        "contacts" => sync_and_print_contacts().await?,
        _ => {
            println!("Invalid option!\n");
            print_options();
        }
    };

    Ok(())
}

