extern crate core;
extern crate qrcodegen;

use qrcodegen::QrCode;
use qrcodegen::QrCodeEcc;

use base64::decode;
use presage::manager::{Manager, ReceivingMode, Registered};
use std::env;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use uuid::Uuid;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::ops::RangeFull;
use futures::channel::oneshot;
use futures::{future, StreamExt};
use presage::libsignal_service::configuration::SignalServers;
use presage::libsignal_service::content::ContentBody;
use presage::libsignal_service::provisioning::SecondaryDeviceProvisioning::Url as PresageUrl;
use presage::store::Thread;
use image::Luma;
use url::Url as ExternalUrl;
use tokio::runtime::Runtime;

fn find_account_uuid(phone_number: &str) -> Option<Uuid> {
    let mut file = File::open("/Users/michalinahytrek/Documents/Signal_client/signal_client/registration/accounts.json").expect("Unable to open file");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Unable to read file");

    let json: Value = serde_json::from_str(&data).expect("Unable to parse JSON");
    if let Some(accounts) = json["accounts"].as_array() {
        for account in accounts {
            if account["number"] == phone_number {
                return Uuid::parse_str(account["uuid"].as_str().unwrap()).ok();
            }
        }
    }
    None
}

fn generate_qr_code(text: &str) {
    let qr = QrCode::encode_text(text, QrCodeEcc::Medium).unwrap();

    let border = 4; // Liczba pikseli obramowania
    let white_block = '\u{2588}';
    let black_block = '\u{2591}';

    // Wydrukuj kod QR z obramowaniem
    for y in -border..qr.size() + border {
        for x in -border..qr.size() + border {
            let block = if qr.get_module(x, y) {
                black_block
            } else {
                white_block
            };
            print!("{}{}{}{}", block, block, block, block); // Wydrukuj cztery bloki zamiast jednego
        }
        println!();
    }
}

async fn send_message(arguments: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let recipient = &arguments[2];
    let message = &arguments[3];

    let store = SledStore::open("/tmp/presage-example/", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
    let mut manager = Manager::load_registered(store.clone()).await?;

    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let timestamp = since_the_epoch.as_millis() as u64;

    if let Some(uuid) = find_account_uuid(recipient) {
        let service_address = presage::libsignal_service::ServiceAddress::from(uuid);

        let data_message = presage::proto::DataMessage {
            body: Some(message.parse().unwrap()), timestamp: Some(timestamp),
            ..Default::default()
        };
        println!("Waiting for sending message");
        manager.send_message(
            service_address,
            ContentBody::from(data_message),
            timestamp
        ).await?;

        println!("Wiadomość wysłana do {}: \"{}\"", recipient, message);
    } else {
        eprintln!("Nie znaleziono konta dla numeru odbiorcy.");
    }

    Ok(())
}

async fn receive_and_store_messages() -> Result<(), Box<dyn std::error::Error>> {
    let store = SledStore::open("/tmp/presage-example/", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
    let mut manager = Manager::load_registered(store.clone()).await?;
    let mut messages = manager.receive_messages(ReceivingMode::Forever).await?;
    Ok(())
}

async fn link_account(arguments: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let account_name = &arguments[2];

    let store = SledStore::open("/tmp/presage-example", MigrationConflictStrategy::Drop, OnNewIdentity::Trust)?;

    let (tx, rx) = oneshot::channel();
    let (manager, err) = future::join(
        Manager::link_secondary_device(
            store,
            SignalServers::Production,
            account_name.clone().into(),
            tx,
        ),
        async move {
            match rx.await {
                Ok(url) => {
                    generate_qr_code(&url.to_string());
                    println!("Show URL {} as QR code to user", url);
                }
                Err(e) => println!("Error linking device: {}", e),
            }
        },
    ).await;

    Ok(())
}

async fn show_messages(arguments: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let contact = &arguments[2];
    if let Some(uuid) = find_account_uuid(contact) {
        println!("Znaleziono uuid");
        let store = SledStore::open("/tmp/presage-example/", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
        let mut manager = Manager::load_registered(store.clone()).await?;

        let thread = Thread::Contact(uuid);
        let messages = manager.messages(&thread, RangeFull)?;

        for message in messages {
            println!("{:?}", message?);
        }
    } else {
        println!("Nie znaleziono kontaktu");
    }

    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let option = &args[1];
    match option.as_str() {
        "send" => send_message(args).await?,
        "account" => link_account(args).await?,
        "receive" => receive_and_store_messages().await?,
        "show" => show_messages(args).await?,
        _ => {}
    };

    Ok(())
}
