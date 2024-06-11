
use presage::{Manager, store::{ContentsStore, StateStore}};
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use std::fs::File;
use serde_json::Value;
use uuid::Uuid;
use std::io::Read;


pub fn find_account_uuid(phone_number: &str) -> Option<Uuid> {
    let mut file = File::open("./registration/contacts.json").expect("Unable to open file");
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

pub fn find_phone_number(uuid: &str) -> Option<String> {
    let mut file = File::open("./registration/contacts.json").expect("Unable to open file");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Unable to read file");

    let json: Value = serde_json::from_str(&data).expect("Unable to parse JSON");
    if let Some(accounts) = json["accounts"].as_array() {
        for account in accounts {
            if account["uuid"] == uuid {
                return account["number"].as_str().map(|s| s.to_string());
            }
        }
    }
    None
}

pub async fn sync_and_print_contacts() -> Result<(), Box<dyn std::error::Error>> {
    let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
    // let mut manager = Manager::load_registered(store.clone()).await?;
    // println!("Czekamy na synchronizacje");
    // manager.sync_contacts().await?;
    // println!("Czekamy na request");
    // manager.request_contacts().await?;

    let contacts_iter = store.contacts()?;
    println!("Iterujemy");
    for contact_result in contacts_iter {
        match contact_result {
            Ok(contact) => println!("{:?}", contact),
            Err(err) => eprintln!("Error retrieving contact: {:?}", err),
        }
    }

    Ok(())
}
