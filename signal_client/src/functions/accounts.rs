use qrcodegen::QrCode;
use qrcodegen::QrCodeEcc;
use futures::channel::oneshot;
use futures::{future};
use presage::manager::Manager;
use presage::libsignal_service::configuration::SignalServers;
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use std::error::Error;


pub fn generate_qr_code(text: &str) {
    let qr = QrCode::encode_text(text, QrCodeEcc::Medium).unwrap();

    let border = 4;
    let white_block = '\u{2588}';
    let black_block = '\u{2591}';

    for y in -border..qr.size() + border {
        for x in -border..qr.size() + border {
            let block = if qr.get_module(x, y) {
                black_block
            } else {
                white_block
            };
            print!("{}{}{}{}", block, block, block, block);
        }
        println!();
    }
}

pub async fn link_account(arguments: Vec<String>) -> Result<(), Box<dyn Error>> {
    if arguments.len() < 3{
        println!("Invalid arguments. Please try:");
        println!("  account <account_name>      - Link an account");
    }

    close_previous_session().await?;

    let account_name = &arguments[2];
    let store = SledStore::open("./registration/main", MigrationConflictStrategy::Drop, OnNewIdentity::Trust)?;
    let (tx, rx) = oneshot::channel();
    let (_manager, _err) = future::join(
        Manager::link_secondary_device(
            store,
            SignalServers::Production,
            account_name.clone().into(),
            tx,
        ),
        async move {
            match rx.await {
                Ok(url) => {
                    generate_qr_code(&url.to_string());
                    println!("URL code: {} ", url);
                }
                Err(e) => println!("Error linking device: {}", e),
            }
        },
    ).await;
    Ok(())
}

async fn close_previous_session() -> Result<(), Box<dyn std::error::Error>>{
    match async {
        println!("otwieram sklep");
        let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
        println!("manager");
        let manager = Manager::load_registered(store).await?;
        let registration_data = manager.registration_data();
        println!("not yet");
        let service_address = presage::libsignal_service::ServiceAddress::from(registration_data.service_ids.aci);
        println!("????");
        manager.clear_sessions(&service_address).await?;
        println!("cleared");

        Ok::<(), Box<dyn std::error::Error>>(())
    }.await {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}
