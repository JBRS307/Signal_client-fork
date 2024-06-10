extern crate core;

use base64::decode;
use presage::manager::{Manager, Registered};
use std::env;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use uuid::Uuid;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use futures::channel::oneshot;
use futures::future;
use presage::libsignal_service::configuration::SignalServers;
use presage::libsignal_service::content::ContentBody;


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


// odkomentuj jak zarejestrujesz linkiem z drugiego main


#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Pobierz argumenty z linii komend
    let args: Vec<String> = env::args().collect();

    let sender = &args[1];
    let recipient = &args[2];
    let message = &args[3];

    // let sender = "+48574596401";
    // let recipient = "+48574596401";
    // let message = "Nowa wiadomosc";


    // Inicjalizacja sklepu Sled
    let store = SledStore::open("/tmp/presage-example/", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;


    // Załaduj zarejestrowane konto nadawcy
    let mut manager = Manager::load_registered(store.clone()).await?;

    let mut registration_data = manager.registration_data();
    println!("{:?}", registration_data.device_name);
    println!("{:?}", registration_data.phone_number);
    println!("{:?}", registration_data.service_ids);
    println!("{:?}", registration_data.device_id);

    // Pobierz czas obecny jako timestamp
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let timestamp = since_the_epoch.as_secs();

    println!("{}", timestamp);


    // Utwórz ServiceAddress z PhoneNumber

    if let Some(uuid) = find_account_uuid(recipient) {
        let service_address = presage::libsignal_service::ServiceAddress::from(uuid);
        let service_address = presage::libsignal_service::ServiceAddress::from(uuid);

        let data_message = presage::proto::DataMessage {
            body: Some(message.parse().unwrap()), timestamp: Option::from(timestamp),
            ..Default::default()
        };


        // Wyślij wiadomość
        manager.send_message(
            service_address,
            ContentBody::from(data_message),
            timestamp
        ).await?;

        println!("Wiadomość wysłana od {} do {}: \"{}\"", sender, recipient, message);
    } else {
        eprintln!("Nie znaleziono konta dla numeru odbiorcy.");
    }

    Ok(())
}


//Tworzysz plik, uzywasz tylko raz tego xd

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let store =
//         SledStore::open("/tmp/presage-example", MigrationConflictStrategy::Drop, OnNewIdentity::Trust)?;
//
//     let (mut tx, mut rx) = oneshot::channel();
//     let (manager, err) = future::join(
//         Manager::link_secondary_device(
//             store,
//             SignalServers::Production,
//             "Michalina-phone".into(),
//             tx,
//         ),
//         async move {
//             match rx.await {
//                 Ok(url) => println!("Show URL {} as QR code to user", url),
//                 Err(e) => println!("Error linking device: {}", e),
//             }
//         },
//     )
//         .await;
//
//     Ok(())
// }
