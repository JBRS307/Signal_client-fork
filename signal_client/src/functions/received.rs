use std::ops::RangeFull;
use futures::StreamExt;
use presage::Manager;
use presage::manager::ReceivingMode;
use presage::store::Thread;
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use crate::functions::contacts::{find_account_uuid};
use crate::functions::messages::extract_message_info;


pub async fn receive_and_store_messages() -> Result<(), Box<dyn std::error::Error>> {
    let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
    let mut manager = Manager::load_registered(store.clone()).await?;
    let mut messages = Box::pin(manager.receive_messages(ReceivingMode::Forever).await?);
    while let Some(message) = messages.next().await {
        extract_message_info(&message);
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