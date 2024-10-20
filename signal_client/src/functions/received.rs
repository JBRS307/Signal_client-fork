use std::ops::RangeFull;
use futures::StreamExt;
use presage::libsignal_service::content::ContentBody;
use presage::Manager;
use presage::manager::{ReceivingMode, Registered};
use presage::store::Thread;
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use crate::functions::contacts::{find_account_uuid};
use crate::functions::messages::{extract_last_info, extract_message_info};
use std::pin::pin;


pub async fn receive_and_store_messages() -> Result<(), Box<dyn std::error::Error>> {
    let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
    let mut manager = Manager::load_registered(store.clone()).await?;
    let mut messages = pin!(manager.receive_messages(ReceivingMode::InitialSync).await?);
    while let Some(message) = messages.next().await {
        extract_message_info(&message);
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
                extract_message_info(&msg);
            } else if let Err(err) = message {
                eprintln!("Error processing message: {:?}", err);
            }
        }

    } else {
        println!("No contact found");

    }

    Ok(())
}

// Function to show most recent message for given contact
pub fn show_last_message(contact: &String, manager: &Manager<SledStore, Registered>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(uuid) = find_account_uuid(contact) {
        // let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
        // let manager = Manager::load_registered(store.clone()).await?;
        let thread = Thread::Contact(uuid);
        let messages = manager.messages(&thread, RangeFull)?;

        for message in messages.into_iter().rev() {
            if let Ok(msg) = message {
                if let ContentBody::DataMessage(_) = msg.body {
                    extract_last_info(&msg);
                    return Ok(());
                }
            } else if let Err(err) = message {
                eprintln!("Error processing message: {:?}", err);
            }
        }
        println!("No DataMessage found for the contact");

    } else {
        println!("No contact found");
    }

    Ok(())
}

