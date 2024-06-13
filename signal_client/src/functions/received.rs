use std::ops::RangeFull;
use futures::StreamExt;
use presage::Manager;
use uuid::Uuid;
use crate::App;
use presage::manager::ReceivingMode;
use presage::store::{Thread, ContentsStore, Store};
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use crate::functions::contacts::{find_account_uuid};
use crate::functions::messages::extract_message_info;


pub async fn receive_and_store_messages() -> Result<(), Box<dyn std::error::Error>> {
    let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
    let mut manager = Manager::load_registered(store.clone()).await?;
    let mut messages = Box::pin(manager.receive_messages(ReceivingMode::Forever).await?);
    while let Some(message) = messages.next().await {
        extract_message_info(&message, true);
    }
    Ok(())
}


pub async fn show_messages(arguments: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let contact = &arguments[2];
    if let Some(uuid) = find_account_uuid(contact) {
        let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
        let mut manager = Manager::load_registered(store.clone()).await?;
        let thread = Thread::Contact(uuid);
        let messages = manager.messages(&thread, RangeFull)?;

        for message in messages {
            if let Ok(msg) = message {
                extract_message_info(&msg, true);
            } else if let Err(err) = message {
                eprintln!("Error processing message: {:?}", err);
            }
        }

    } else {
        println!("No contact found");

    }

    Ok(())
}

pub async fn show_last_message(contact: &String) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(uuid) = find_account_uuid(contact) {
        let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
        let manager = Manager::load_registered(store.clone()).await?;
        let thread = Thread::Contact(uuid);
        let messages = manager.messages(&thread, RangeFull)?;
        if let Some(last_message) = messages.last() {
            if let Ok(msg) = last_message {
                extract_message_info(&msg, true);
            } else if let Err(err) = last_message {
                eprintln!("Error processing message: {:?}", err);
            }
        } else {
            println!("No messages found for the contact");
        }

    } else {
        println!("No contact found");
    }

    Ok(())
}

pub async fn get_contact_messages(contact: &str) -> Result<Vec<(String, String, u64)>, Box<dyn std::error::Error>> {
    let mut messages = Vec::new();

    if let Some(uuid) = find_account_uuid(contact) {
        let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
        let manager = Manager::load_registered(store.clone()).await?;

        let thread = Thread::Contact(uuid);
        let manager_messages = manager.messages(&thread, RangeFull)?;

        for message in manager_messages {
            if let Ok(msg) = message {
                if let Some(info) = extract_message_info(&msg, false) {
                    let (sender, body, timestamp) = info;
                    let body_string = body.to_string();
                    let modified_info = (sender, body_string, timestamp); 
                    messages.push(modified_info);
                }
            } else if let Err(err) = message {
                eprintln!("Error processing message: {:?}", err);
            }
        }
    } else {
        println!("No contact found");
    }

    Ok(messages)
}