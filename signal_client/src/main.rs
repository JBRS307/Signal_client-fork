extern crate core;
extern crate qrcodegen;

mod functions;
use std::env;
use crate::functions::accounts::link_account;
use crate::functions::contacts::sync_and_print_contacts;
use crate::functions::received::{receive_and_store_messages, show_messages};
use crate::functions::sending::{send_message};
use crate::functions::ui::start_tui;
use crate::functions::group::sync_and_print_groups;

fn print_options() {
    println!("Please use one of the following options:");
    println!("  send <recipient> <message>  - Send a message to a recipient");
    println!("  account <account_name>      - Link an account");
    println!("  receive                     - Receive and store messages");
    println!("  contacts                    - Show all contacts");
    println!("  show <contact>              - Show messages for a contact");
    println!("  tui                         - Start the terminal UI");
}

pub struct App {
    contacts: Vec<String>,
    messages: Vec<(String, String, u64, String)>,
    selected_contact: Option<usize>,
    name: String,
}

impl App {
    fn new(contacts: Vec<String>, name: String) -> App {
        App {
            contacts,
            messages: vec![],
            selected_contact: None,
            name,
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("WELCOME TO OUR SIGNAL-CLIENT!\n");
        print_options();
        return Ok(());
    }

    let option = &args[1];
    match option.as_str() {
        "send" =>   {
            send_message(args).await?;
            receive_and_store_messages().await?
        }
        "account" => link_account(args).await?,
        "receive" => receive_and_store_messages().await?,
        "show" => show_messages(args).await?,
        "contacts" => sync_and_print_contacts().await?,
        "groups" => sync_and_print_groups().await?,
        "tui" => start_tui().await?,
        _ => {
            println!("Invalid option!\n");
            print_options(); }
    };
    Ok(())
}