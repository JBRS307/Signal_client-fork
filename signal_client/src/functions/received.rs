use std::ops::RangeFull;
use futures::StreamExt;
use presage::libsignal_service::content::ContentBody;
use presage::Manager;
use presage::manager::ReceivingMode;
use presage::store::{Thread};
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use crate::functions::contacts::{find_account_uuid};
use crate::functions::messages::{extract_last_info, extract_message_info};


pub async fn receive_and_store_messages() -> Result<(), Box<dyn std::error::Error>> {
    let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
    let mut manager = Manager::load_registered(store.clone()).await?;
    let mut messages = Box::pin(manager.receive_messages(ReceivingMode::InitialSync).await?);
    while let Some(message) = messages.next().await {
        extract_message_info(&message, true);
    }
    Ok(())
}


pub async fn show_messages(arguments: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let contact = &arguments[2];
    if let Some(uuid) = find_account_uuid(contact) {
        let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
        let manager = Manager::load_registered(store.clone()).await?;
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

pub async fn show_last_message(contact: &String, store: &SledStore) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(uuid) = find_account_uuid(contact) {
        // let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
        let manager = Manager::load_registered(store.clone()).await?;
        let thread = Thread::Contact(uuid);
        let messages = manager.messages(&thread, RangeFull)?;

        for message in messages.into_iter().rev() {
            if let Ok(msg) = message {
                if let ContentBody::DataMessage(..) = msg.body {
                    extract_last_info(&msg);
                    return Ok(());
                }
            } else if let Err(err) = message {
                eprintln!("Error processing message: {:?}", err);
            }
        }
        println!("No DataMessage found for the contact");

    } else {
        println!("No message found");
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