use qrcodegen::QrCode;
use qrcodegen::QrCodeEcc;
use futures::channel::oneshot;
use futures::{future};
use presage::manager::Manager;
use presage::libsignal_service::configuration::SignalServers;
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use std::error::Error;
use std::fs;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use super::paths;


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

async fn close_previous_session() -> Result<(), Box<dyn std::error::Error>> {
    match async {
        let path = Path::new("./registration/main");
        if path.exists() && path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(&path)?;
                } else if path.is_dir() {
                    fs::remove_dir_all(&path)?;
                }
            }
        }

        let contacts_path = Path::new("./registration/contacts.json");
        if contacts_path.exists() {
            let mut file = File::create(contacts_path).await?;
            file.write_all(b"{}").await?;
        }

        Ok::<(), Box<dyn std::error::Error>>(())
    }.await {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}

pub async fn print_current_user() -> Result<(), Box<dyn std::error::Error>> {
    let store = SledStore::open(paths::DATABASE, MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
    let mut manager = Manager::load_registered(store.clone()).await?;
    
    let profile = manager.retrieve_profile().await?;

    print!("Name: ");
    if let Some(name) = profile.name {
        print!("{} ", name.given_name);
        if let Some(lastname) = name.family_name {
            print!("{}", lastname);
        }
    }
    println!("");

    print!("About: ");
    if let Some(about) = profile.about {
        print!("{}", about);
    }
    println!("");

    Ok(())

}