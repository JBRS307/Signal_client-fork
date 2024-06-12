use presage::{Manager, store::{ContentsStore, StateStore}};
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use std::fs::File;
use uuid::Uuid;
use std::io::{Read, Write};
use serde_json::{json, Value};
use std::fs::OpenOptions;
use presage::libsignal_service::models::Contact;

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

fn contact_exists(json: &Value, uuid: &str) -> bool {
    if let Some(accounts) = json["accounts"].as_array() {
        for account in accounts {
            if account["uuid"] == uuid {
                return true;
            }
        }
    }
    false
}

fn add_contacts_to_json(contact: Contact) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new().read(true).write(true).create(true).open("./registration/contacts.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    println!("nie umie czytac xd");
    let mut json: Value = serde_json::from_str(&data)?;
    println!("Tiutaj jest problem");
    if !json["accounts"].is_array() {
        json["accounts"] = json!([]);
    }
    println!("Contact exist");
    if !contact_exists(&json, &contact.uuid.to_string()) {
        if let Some(accounts) = json["accounts"].as_array_mut() {
            let new_account = json!({
                "name": contact.name,
                "uuid": contact.uuid.to_string()
            });
            accounts.push(new_account);
        }
    }

    let updated_data = serde_json::to_string_pretty(&json)?;
    file.set_len(0)?;
    file.write_all(updated_data.as_bytes())?;
    Ok(())
}

pub async fn sync_and_print_contacts() -> Result<(), Box<dyn std::error::Error>> {
    let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
    let contacts_iter = store.contacts()?;
    for contact_result in contacts_iter {
        match contact_result {
            Ok(contact) => {
                // add_contacts_to_json(contact).expect("Contact not saved");
                println!("{:?}", contact)
            },
            Err(err) => eprintln!("Error retrieving contact: {:?}", err),
        }
    }

    Ok(())
}
