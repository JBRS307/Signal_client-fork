use presage::{Manager, store::{ContentsStore, StateStore}};
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use std::fs::File;
use uuid::Uuid;
use std::io::{Read, Write};
use serde_json::{json, Value};
use std::fs::OpenOptions;
use presage::libsignal_service::models::Contact;
use crate::functions::received::show_last_message;

pub fn find_account_uuid(name: &str) -> Option<Uuid> {
    let mut file = File::open("./registration/contacts.json").expect("Unable to open file");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Unable to read file");

    let json: Value = serde_json::from_str(&data).expect("Unable to parse JSON");
    if let Some(accounts) = json["accounts"].as_array() {
        for account in accounts {
            if account["name"] == name {
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

pub fn find_name(uuid: &str) -> Option<String> {
    let mut file = File::open("./registration/contacts.json").expect("Unable to open file");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Unable to read file");

    let json: Value = serde_json::from_str(&data).expect("Unable to parse JSON");
    if let Some(accounts) = json["accounts"].as_array() {
        for account in accounts {
            if account["uuid"] == uuid {
                return account["name"].as_str().map(|s| s.to_string());
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
    let mut data = String::new();
    let mut json: Value;

    // Read existing content or initialize new JSON
    if let Ok(mut file) = File::open("./registration/contacts.json") {
        file.read_to_string(&mut data)?;
        json = if data.trim().is_empty() {
            json!({"accounts": [], "version": 2})
        } else {
            serde_json::from_str(&data)?
        };
    } else {
        json = json!({"accounts": [], "version": 2});
    }

    if !json["accounts"].is_array() {
        json["accounts"] = json!([]);
    }

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
    let mut file = OpenOptions::new().write(true).truncate(true).create(true).open("./registration/contacts.json")?;
    file.write_all(updated_data.as_bytes())?;
    Ok(())
}

pub async fn sync_and_print_contacts() -> Result<(), Box<dyn std::error::Error>> {
    let contacts = sync_and_get_contacts().await?;
    for contact in contacts {
        println!("{}", contact);
    }

    Ok(())
}

pub async fn sync_and_get_contacts() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
    // let mut manager = Manager::load_registered(store.clone()).await?;
    let contacts_iter = store.contacts()?;
    
    let mut contact_names = Vec::new();
    for contact_result in contacts_iter {
        match contact_result {
            Ok(contact) => {
                println!("{}", contact.name );
                show_last_message(&contact.name);
                println!("-------------------");
                contact_names.push(contact.name.clone());
                if let Err(e) = add_contacts_to_json(contact) {
                    eprintln!("Contact not saved: {:?}", e);
                }
            },
            Err(err) => eprintln!("Error retrieving contact: {:?}", err),
        }
    }

    Ok(contact_names)
}