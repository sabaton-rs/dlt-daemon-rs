use dlt_core::{
    self,
    dlt::{MessageConfig, StorageHeader},
};

pub fn log_string(log: String) {
    let conf = MessageConfig {
        version: 1,
        counter: todo!(),
        endianness: todo!(),
        ecu_id: todo!(),
        session_id: todo!(),
        timestamp: todo!(),
        payload: todo!(),
        extended_header_info: Some(dlt_core::dlt::ExtendedHeaderConfig {
            message_type: dlt_core::dlt::MessageType::Log(dlt_core::dlt::LogLevel::Debug),
            app_id: String::from("ABCD"),
            context_id: String::from("ABCD"),
        }),
    };
    let storage_header = StorageHeader {
        timestamp: todo!(),
        ecu_id: todo!(),
    };
    let message = dlt_core::dlt::Message::new(conf, Some(storage_header));
}
