use std::{ffi::c_char, os::fd::RawFd};

use libdlt::error::DltUserError;


enum LogState {
    Unknown,
    Disconnected,
    Connected,
}

impl Default for LogState {
    fn default() -> Self {
        Self::Unknown
    }
}

pub struct DltUser {
    ecu_id : u32,
    app_id : u32,
    // handle to fifo for sending to dlt-daemon
    log_handle : Option<RawFd>,
    // handle to fifo for receiving from dlt-daemon
    user_handle : Option<RawFd>,
    // if we are logging to file
    logging_to_file : bool,
    // overflow counter
    overflow : Option<u32>,
    application_description : String,
    verbose_mode : bool,
    use_extended_header_for_non_verbose : bool, /**< Use extended header for non verbose: 1 enabled, 0 disabled */
    with_session_id : bool,                    /**< Send always session id: 1 enabled, 0 disabled */
    with_timestamp : bool,                     /**< Send always timestamp: 1 enabled, 0 disabled */
    with_ecu_id : bool,
    
    enable_local_print : bool,
    local_print_mode : LocalPrintMode,

    log_state : LogState,
    contexts : Vec<Context>,

    initial_log_levels : Vec<InitialLogLevel>,

}

impl DltUser {
    pub fn new(ecu_id: [c_char;4], app_id:[c_char;4], description: String) -> Result<Self,DltUserError> {
        let ecu_id = [ecu_id[0] as u8,ecu_id[1] as u8, ecu_id[2] as u8, ecu_id[3] as u8];
        let app_id = [app_id[0] as u8,app_id[1] as u8, app_id[2] as u8, app_id[3] as u8];
        let dlt_user = DltUser {
            ecu_id: u32::from_ne_bytes(ecu_id),
            app_id: u32::from_ne_bytes(app_id),
            log_handle: None,
            user_handle: None,
            logging_to_file: false,
            overflow: None,
            application_description: description,
            verbose_mode: true,
            use_extended_header_for_non_verbose: true,
            with_session_id:true,
            with_timestamp: true,
            with_ecu_id: true,
            enable_local_print: false,
            local_print_mode: LocalPrintMode::Unset,
            log_state: LogState::default(),
            contexts: Vec::new(),
            initial_log_levels: Vec::new(),
        };

        Ok(dlt_user)
    }
}

enum LocalPrintMode {
    Unset,
    Automatic,
    ForceOn,
    ForceOff,
}
impl Default for LocalPrintMode {
    fn default() -> Self {
        Self::Unset
    }
}

struct InitialLogLevel {
    app_id : u32,
    context_id : u32,
    log_level : i8,
}

struct Context {
    context_id : u32,
    log_level : i8,
    trace_status : i8,
    message_counter : u8,
    description : String,
}




#[cfg(test)]
mod tests {
    use super::*;

}
