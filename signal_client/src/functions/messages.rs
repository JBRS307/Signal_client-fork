use presage::libsignal_service::proto::DataMessage;
use presage::proto::{data_message, sync_message};
use presage::proto::unidentified_sender_message::message;
use presage::{libsignal_service::content::Content};
use presage::libsignal_service::content::ContentBody;
use presage::libsignal_service::content::Metadata;
use chrono::{DateTime, NaiveDateTime, Utc};
use colored::Colorize;
use crate::functions::contacts::{find_name, find_phone_number};

fn extract_message<'a>(metadata: &Metadata, msg: &'a DataMessage) -> (String, u64, &'a str, String) {
    let sender_aci = metadata.sender.uuid.to_string();
    let message_timestamp = msg.timestamp();
    let message_body = msg.body();
    let message_date = format_timestamp(message_timestamp);
    (sender_aci, message_timestamp, message_body, message_date)
}

pub fn extract_message_info(content: &Content) -> Option<(String, &str, u64)> {
    if let Content {
        metadata,
        body: ContentBody::DataMessage(sync_message),
    } = content {
        let (sender_aci, message_timestamp, message_body, message_date) = extract_message(metadata, sync_message);
        if let Some(name) = find_name(sender_aci.as_str()){
            println!("Sender: {:?} \nMessage: {:?} \nTime: {:?} \n", name , message_body, message_date);
        } else{
            println!("Sender: {:?} \nMessage: {:?} \nTime: {:?} \n", sender_aci , message_body, message_date);
        }
        return Some((sender_aci, message_body, message_timestamp));
    } else if let Content {
        metadata,
        body: ContentBody::SynchronizeMessage(sync_message)
    } = content {
        let data_message = match &sync_message.sent {
            Some(sent) => {
                match &sent.message {
                    Some(msg) => msg,
                    None => return None,
                }
            },
            None => return None,
        };
        
        let (sender_aci, message_timestamp, message_body, message_date) = extract_message(metadata, data_message);
        if message_body == "" {
            return None;
        }
        if let Some(name) = find_name(sender_aci.as_str()){
            println!("Sender: {:?} \nMessage: {:?} \nTime: {:?} \n", name , message_body, message_date);
        } else{
            println!("Sender: {:?} \nMessage: {:?} \nTime: {:?} \n", sender_aci , message_body, message_date);
        }
        return Some((sender_aci, message_body, message_timestamp));
    }
    println!("Nie datamessage");
    None
}

pub fn extract_last_info(content: &Content) -> Option<(String, &str, u64)> {
    if let Content {
        metadata,
        body: ContentBody::DataMessage(sync_message),
    } = content {
        let sender_aci = metadata.sender.uuid.to_string();
        let message_timestamp = sync_message.timestamp();
        let message_body = sync_message.body();
        if let Some(name) = find_name(sender_aci.as_str()){
            println!("Last Message: {:?} \n", message_body);
        } else{
        }
        return Some((sender_aci, message_body, message_timestamp));
    }
    println!("Nie datamessage");
    None
}

pub fn format_timestamp(timestamp: u64) -> String {
    let timestamp = timestamp / 1000;
    let naive = NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).expect("Invalid timestamp");
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime.format("%d.%m.%Y %H:%M:%S").to_string()
}