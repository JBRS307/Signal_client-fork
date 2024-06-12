use presage::libsignal_service::content::Content;
use presage::libsignal_service::content::ContentBody::DataMessage;
use chrono::{DateTime, NaiveDateTime, Utc};
use crate::functions::contacts::{find_name, find_phone_number};

pub fn extract_message_info(content: &Content) -> Option<(String, &str, u64)> {
    if let Content {
        metadata,
        body: DataMessage(sync_message),
    } = content {
        let sender_aci = metadata.sender.uuid.to_string();
        let message_timestamp = sync_message.timestamp();
        let message_body = sync_message.body();
        let message_date = format_timestamp(message_timestamp);
        // if let Some(name) = find_name(sender_aci.as_str()){
        //     println!("Sender: {:?} \nMessage: {:?} \nTime: {:?} \n", name , message_body, message_date);
        // } else{
        //     println!("Sender: {:?} \nMessage: {:?} \nTime: {:?} \n", sender_aci , message_body, message_date);
        // }
        return Some((sender_aci, message_body, message_timestamp));
    }
    None
}

pub fn format_timestamp(timestamp: u64) -> String {
    let timestamp = timestamp / 1000;
    let naive = NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).expect("Invalid timestamp");
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime.format("%d.%m.%Y %H:%M:%S").to_string()
}