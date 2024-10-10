use std::time::{SystemTime, UNIX_EPOCH};
use presage::libsignal_service::content::ContentBody;
use presage::Manager;
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use crate::functions::contacts::find_account_uuid;

pub async fn send_message(arguments: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if arguments.len() < 4{
        println!("Invalid arguments. Please try:");
        println!("  send <recipient> <message>   - Send a message to a recipient");
    }
    let recipient = &arguments[2];
    let message = &arguments[3];

    let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
    let mut manager = Manager::load_registered(store.clone()).await?;

    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let timestamp = since_the_epoch.as_millis() as u64;

    if let Some(uuid) = find_account_uuid(recipient) {
        let service_address = presage::libsignal_service::ServiceAddress::new_aci(uuid);

        let data_message = presage::proto::DataMessage {
            body: Some(message.parse().unwrap()), timestamp: Some(timestamp),
            ..Default::default()
        };
        println!("Waiting for sending message");
        manager.send_message(
            service_address,
            ContentBody::from(data_message),
            timestamp
        ).await?;

        println!("Message send to {}: \"{}\"", recipient, message);
    } else {
        eprintln!("No such contact");
    }

    Ok(())
}