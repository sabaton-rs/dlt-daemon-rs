#[derive(Debug)]

pub enum UserMessageType {
    Log,
    RegisterApplication,
    UnRegisterApplication,
    RegisterContext,
    UnRegisterContext,
    LogLevel,
    Injection,
    OverFlow,
    AppLlTs,
    LogShm,
    LogMode,
    LogState,
    Marker,
    NotSupported,
}
impl From<UserMessageType> for u32 {
    fn from(value: UserMessageType) -> u32 {
        match value {
            UserMessageType::Log => 1,
            UserMessageType::RegisterApplication => 2,
            UserMessageType::UnRegisterApplication => 3,
            UserMessageType::RegisterContext => 4,
            UserMessageType::UnRegisterContext => 5,
            UserMessageType::LogLevel => 6,
            UserMessageType::Injection => 7,
            UserMessageType::OverFlow => 8,
            UserMessageType::AppLlTs => 9,
            UserMessageType::LogShm => 10,
            UserMessageType::LogMode => 11,
            UserMessageType::LogState => 12,
            UserMessageType::Marker => 13,
            UserMessageType::NotSupported => 16,
        }
    }
}
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct UserHeader {
    pattern: [u8; 4],
    message_type: u32,
}
impl UserHeader {
    pub fn new(message_type: UserMessageType) -> Self {
        UserHeader {
            pattern: ['D' as u8, 'U' as u8, 'H' as u8, 1],
            message_type: message_type.into(),
        }
    }
}
pub mod user_control_message {
    use std::{mem, process};

    use crate::DltUserInner;

    pub(crate) fn opt_string_to_u8_4(str: Option<String>) -> [u8; 4] {
        let mut result = [0u8; 4];
        if let Some(s) = str {
            for (i, c) in s.chars().take(4).enumerate() {
                result[i] = c as u8;
            }
        }
        result
    }
    #[derive(Clone, Copy, Debug)]
    #[repr(C, packed)]
    pub(crate) struct RegisterApplication {
        pub app_id: [u8; 4],
        pub pid: u32,
        pub description_length: u32,
    }
    impl RegisterApplication {
        pub fn new(dltuserinner: &DltUserInner) -> Self {
            RegisterApplication {
                app_id: opt_string_to_u8_4(dltuserinner.app_id.clone()),
                pid: process::id(),
                description_length: dltuserinner.application_description.len() as u32,
            }
        }
    }
    #[repr(C, packed)]
    pub struct UnregisterApplication {
        app_id: [u8; 4],
        pid: u32,
    }
    impl UnregisterApplication {
        fn new(dltuserinner: &DltUserInner) -> Self {
            UnregisterApplication {
                app_id: opt_string_to_u8_4(dltuserinner.app_id.clone()),
                pid: process::id(),
            }
        }
    }
    #[repr(C, packed)]
    pub struct RegisterContext {
        app_id: [u8; 4],
        context_id: [u8; 4],
        log_level_pos: i32,
        log_level: i32,
        trace_status: i8,
        pid: u32,
        description_length: u32,
    }
    impl RegisterContext {
        fn new(dltuserinner: &DltUserInner) -> Self {
            RegisterContext {
                app_id: opt_string_to_u8_4(dltuserinner.app_id.clone()),
                context_id: todo!(),
                log_level_pos: 0,
                log_level: 0,
                trace_status: todo!(),
                pid: process::id(),
                description_length: todo!(),
            }
        }
    }

    #[repr(C, packed)]
    pub struct UnRegisterContext {
        app_id: [u8; 4],
        context_id: [u8; 4],
        pid: u32,
    }
    impl UnRegisterContext {
        fn new(dltuserinner: &DltUserInner) -> Self {
            UnRegisterContext {
                app_id: opt_string_to_u8_4(dltuserinner.app_id.clone()),
                context_id: todo!(),
                pid: process::id(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dlt_user, user_header::user_control_message::RegisterApplication};

    #[test]
    fn user_header() {
        let user_header = UserHeader::new(UserMessageType::RegisterApplication);
        assert_eq!(user_header.pattern, ['D' as u8, 'U' as u8, 'H' as u8, 1]);
        assert!(user_header.message_type == 2);
    }
    #[test]
    fn register_application() {
        const CONFIG: &str = "../libdlt/testdata/daemon.conf";
        let dlt_user = dlt_user();
        dlt_user.dlt_register_app("ECU1", "THIS IS FIRST TEST");
        let register_application = RegisterApplication::new(&dlt_user.inner.lock().unwrap());
        assert_eq!(register_application.app_id, [69, 67, 85, 49]);
        assert!(register_application.description_length == 18);
    }
}
