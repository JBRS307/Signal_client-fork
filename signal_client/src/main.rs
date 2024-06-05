use presage::manager::Manager;
use std::env;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use uuid::Uuid;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use presage::libsignal_service::content::ContentBody;

fn find_account_uuid(phone_number: &str) -> Option<Uuid> {
    let mut file = File::open("/System/Volumes/Data/Users/michalinahytrek/.local/share/signal-cli/data/accounts.json").expect("Unable to open file");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Unable to read file");

    let json: Value = serde_json::from_str(&data).expect("Unable to parse JSON");
    if let Some(accounts) = json["accounts"].as_array() {
        for account in accounts {
            if account["number"] == phone_number {
                println!("{}", account["uuid"]);
                return Uuid::parse_str(account["uuid"].as_str().unwrap()).ok();
            }
        }
    }
    None
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Pobierz argumenty z linii komend
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Użycie: wyslij nadawca odbiorca \"wiadomosc\"");
        return Ok(());
    }

    let sender = &args[1];
    let recipient = &args[2];
    let message = &args[3];


    // Inicjalizacja sklepu Sled
    let store = SledStore::open("/System/Volumes/Data/Users/michalinahytrek/.local/share/signal-cli/dat/accounts.json", MigrationConflictStrategy::Drop, OnNewIdentity::Trust)?;

    // Załaduj zarejestrowane konto nadawcy
    let mut manager = Manager::load_registered(store.clone()).await?;

    // Pobierz czas obecny jako timestamp
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let timestamp = since_the_epoch.as_secs();

    // Utwórz ServiceAddress z PhoneNumber

    if let Some(uuid) = find_account_uuid(recipient) {
        let service_address = presage::libsignal_service::ServiceAddress::from(uuid);
        println!("{}", "Tu jest problem2" );
        let data_message = presage::proto::DataMessage {
            body: Some(message.parse().unwrap()),
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
