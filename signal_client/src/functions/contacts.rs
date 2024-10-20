use presage::{manager::Registered, proto::data_message::contact, store::{ContentsStore, StateStore}, Manager};
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use std::fs::{self, File};
use uuid::Uuid;
use std::io::{Read, Write};
use serde_json::{json, Value};
use std::fs::OpenOptions;
use colored::Colorize;
use presage::libsignal_service::models::Contact;
use crate::functions::received::show_last_message;
use crate::functions::accounts::load_registered_user;

use super::paths;

pub fn find_account_uuid(name: &str) -> Option<Uuid> {
    let data = fs::read_to_string(paths::CONTACTS).expect("Unable to open file!");

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
    let data = fs::read_to_string(paths::CONTACTS).expect("Unable to open file!");

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
    let data = fs::read_to_string(paths::CONTACTS).expect("Unable to open file!");

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
    let mut json: Value;

    // Read existing content or initialize new JSON
    if let Ok(data) = fs::read_to_string(paths::CONTACTS) {
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
    fs::write(paths::CONTACTS, updated_data)?;
    Ok(())
}

// Wrapper function to sync contacts
pub async fn sync_contacts(manager: &mut Manager<SledStore, Registered>) -> Result<(), Box<dyn std::error::Error>> {
    manager.sync_contacts().await?;
    Ok(())
}

// Function to print contacts with last messages using manager
pub fn print_contacts(manager: &Manager<SledStore, Registered>) -> Result<(), Box<dyn std::error::Error>> {
    let contacts_iter = manager.store().contacts()?;
    for contact_result in contacts_iter {
        match contact_result {
            Ok(contact) => {
                println!("{}", contact.name.blue() );
                show_last_message(&contact.name, manager)?;
                println!("-------------------");
                if let Err(e) = add_contacts_to_json(contact) {
                    eprintln!("Contact not saved: {:?}", e);
                }
            },
            Err(err) => eprintln!("Error retrieving contact: {:?}", err),
        }
    }
    Ok(())
}

// Function to sync and print contacts for compatibility with current ui
pub async fn sync_and_print_contacts() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = load_registered_user().await?;
    sync_contacts(&mut manager).await?;
    print_contacts(&manager)?;
    Ok(())
}
