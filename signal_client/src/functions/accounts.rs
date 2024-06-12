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
