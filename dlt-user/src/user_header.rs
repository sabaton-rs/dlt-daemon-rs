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
const DLT_USER_LOG_LEVEL_NOT_SET: i8 = -2;
const DLT_USER_TRACE_STATUS_NOT_SET: i8 = -2;

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
    use std::process;

    use crate::{Context, DltUserInner};

    use super::{DLT_USER_LOG_LEVEL_NOT_SET, DLT_USER_TRACE_STATUS_NOT_SET};

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
    #[derive(Clone, Copy, Debug)]
    #[repr(C, packed)]
    pub(crate) struct UnRegisterApplication {
        pub app_id: [u8; 4],
        pub pid: u32,
    }
    impl UnRegisterApplication {
        pub fn new(dltuserinner: &DltUserInner) -> Self {
            UnRegisterApplication {
                app_id: opt_string_to_u8_4(dltuserinner.app_id.clone()),
                pid: process::id(),
            }
        }
    }
    #[repr(C, packed)]
    pub struct RegisterContext {
        pub app_id: [u8; 4],
        pub context_id: [u8; 4],
        pub log_level_pos: i32,
        pub log_level: i8,
        pub trace_status: i8,
        pub pid: u32,
        pub description_length: u32,
    }
    impl RegisterContext {
        pub fn new(dltuserinner: &DltUserInner, context: &Context) -> Self {
            RegisterContext {
                app_id: opt_string_to_u8_4(dltuserinner.app_id.clone()),
                context_id: context.store.inner.context_id,
                log_level_pos: 0,
                log_level: DLT_USER_LOG_LEVEL_NOT_SET,
                trace_status: DLT_USER_TRACE_STATUS_NOT_SET,
                pid: process::id(),
                description_length: context.store.inner.description.clone().len() as u32,
            }
        }
    }

    #[repr(C, packed)]
    pub struct UnRegisterContext {
        pub app_id: [u8; 4],
        pub context_id: [u8; 4],
        pub pid: u32,
    }
    impl UnRegisterContext {
        pub fn new(dltuserinner: &DltUserInner, context: &Context) -> Self {
            UnRegisterContext {
                app_id: opt_string_to_u8_4(dltuserinner.app_id.clone()),
                context_id: context.store.inner.context_id,
                pid: process::id(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::process;

    use libdlt::error::DltError;

    use super::*;
    use crate::{
        dlt_user,
        user_header::user_control_message::{
            RegisterApplication, RegisterContext, UnRegisterApplication, UnRegisterContext,
        },
    };

    #[test]
    fn user_header() {
        let user_header = UserHeader::new(UserMessageType::RegisterApplication);
        assert_eq!(user_header.pattern, ['D' as u8, 'U' as u8, 'H' as u8, 1]);
        assert!(user_header.message_type == 2);
    }
    #[test]
    fn register_application() {
        let dlt_user = dlt_user();
        dlt_user.register_app("ECU1", "THIS IS FIRST TEST");
        let register_application = RegisterApplication::new(&dlt_user.inner.lock().unwrap());
        assert_eq!(register_application.app_id, [69, 67, 85, 49]);
        assert!(register_application.description_length == 18);
    }

    #[test]
    fn register_context() {
        let dlt_user = dlt_user();
        let context_id = "CON1";
        let description = "First Context";
        let context = dlt_user
            .new_context(context_id, description)
            .ok_or(DltError::DltReturnWrongParameter)
            .unwrap();
        let registered_context = RegisterContext::new(&dlt_user.inner.lock().unwrap(), &context);
        assert_eq!(registered_context.app_id, [0, 0, 0, 0]);
        assert_eq!(
            registered_context.context_id,
            ['C' as u8, 'O' as u8, 'N' as u8, '1' as u8]
        );
        assert!(registered_context.description_length == 13);
        assert_eq!(registered_context.log_level, DLT_USER_LOG_LEVEL_NOT_SET);
        assert!(registered_context.log_level_pos == 0);
        assert_eq!(
            registered_context.trace_status,
            DLT_USER_TRACE_STATUS_NOT_SET
        );
        assert!(registered_context.pid == process::id());
    }
    #[test]
    fn unregister_context() {
        let dlt_user = dlt_user();
        let registered_context = dlt_user.register_context("CON1", "First Context").unwrap();
        let unregistered_context =
            UnRegisterContext::new(&dlt_user.inner.lock().unwrap(), &registered_context);
        assert_eq!(unregistered_context.app_id, [0, 0, 0, 0]);
        assert_eq!(unregistered_context.context_id, [67, 79, 78, 49]);
        assert!(unregistered_context.pid == process::id());
    }
    #[test]
    fn unregister_application() {
        let dlt_user = dlt_user();
        let unregister_application = UnRegisterApplication::new(&dlt_user.inner.lock().unwrap());
        assert_eq!(unregister_application.app_id, [0, 0, 0, 0]);
        assert!(unregister_application.pid == process::id());
    }
}
