use std::{fs::{self, File, OpenOptions}, io::{Read, Write}};

use super::paths;

use presage::libsignal_service::zkgroup::GroupMasterKeyBytes;
use presage::libsignal_service::groups_v2::Group;
use presage::store::ContentsStore;
use presage_store_sled::{OnNewIdentity, MigrationConflictStrategy, SledStore};
use colored::Colorize;
use serde_json::{json, Value};

fn group_exists(json: &Value, key: &GroupMasterKeyBytes) -> bool {
    if let Some(groups) = json["groups"].as_array() {
        for group in groups {
            if group["master_key"].as_array().unwrap() == key {
                return true;
            }
        }
    }
    false
}

pub fn add_group_to_json(key: &GroupMasterKeyBytes, group: &Group) -> Result<(), Box<dyn std::error::Error>> {
    let mut json: Value;
    if let Ok(data) = fs::read_to_string(paths::GROUPS) {
        json = if data.trim().is_empty() {
            json!({"groups": [], "version": 2})
        } else {
            serde_json::from_str(&data)?
        };
    } else {
        json = json!({"groups": [], "version": 2});
    }

    if !json["groups"].is_array() {
        json["groups"] = json!([]);
    }

    if !group_exists(&json, key) {
        if let Some(groups) = json["groups"].as_array_mut() {
            let new_group = json!({
                "title": group.title,
                "master_key": key,
            });
            groups.push(new_group);
        }
    }

    let updated_data = serde_json::to_string_pretty(&json)?;
    fs::write(paths::GROUPS, updated_data)?;
    Ok(())
}


pub async fn sync_and_print_groups() -> Result<(), Box<dyn std::error::Error>> {
    let store = SledStore::open(paths::DATABASE, MigrationConflictStrategy::BackupAndDrop,OnNewIdentity::Trust)?;
    let groups_iter = store.groups()?;

    for group_res in groups_iter {
        match group_res {
            Ok((_, group)) => {
                println!("{}", group.title.blue());
            },
            Err(e) => eprintln!("{:?}", e),
        };
    }
    Ok(())
}