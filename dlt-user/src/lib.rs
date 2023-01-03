use std::{ffi::c_char, os::fd::RawFd, sync::{Mutex, Once, Arc}, mem::MaybeUninit, thread::JoinHandle};
use async_std::channel::{self, Sender};
use dlt_core::dlt::{Message, MessageConfig, PayloadContent, StorageHeader, DltTimeStamp};
use libdlt::error::DltUserError;
use crate::mainloop::mainloop;

pub (crate) mod log;
pub (crate) mod mainloop;

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
    inner : Arc<Mutex<DltUserInner>>,
}

impl DltUser {
    pub fn set_ecu_id(&self, ecu_id: String) {
        self.inner.lock().unwrap().ecu_id = Some(ecu_id);
    }
    pub fn set_app_info(&self, app_id: String, description: String) {
        self.inner.lock().unwrap().app_id = Some(app_id);
        self.inner.lock().unwrap().application_description = Some(description);
    }

    // Create a new Context for logging
    pub fn new_context(&self, context_id : String, description: String) -> Option<Context> {
        self.inner.lock().unwrap().new_context(context_id, description)
    }
}

fn start_async_mainloop(dlt_user_inner : Arc<Mutex<DltUserInner>>) {
    let dlt_user_inner_copy = dlt_user_inner.clone();
    let main = std::thread::spawn(move|| {
        async_std::task::block_on(mainloop(dlt_user_inner_copy))    
    });
    dlt_user_inner.lock().unwrap().mainloop_joinhandle.replace(main);
}


/// Function to retrieve singleton DltUser
pub fn dlt_user() -> &'static DltUser {
    static mut DLT_USER: MaybeUninit<DltUser> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            // Make it
            let dlt_user = DltUser {
                inner: Arc::new(Mutex::new(DltUserInner::new().unwrap())),
            };
            // start the mainloop
            start_async_mainloop(dlt_user.inner.clone());
            // Store it to the static var
            DLT_USER.write(dlt_user);
        });

        // Now we give out a shared reference to the data, which is safe to use
        // concurrently.
        DLT_USER.assume_init_ref()
    }
}

pub struct DltUserInner {
    ecu_id : Option<String>,
    app_id : Option<String>,
    // if we are logging to file
    logging_to_file : bool,
    // overflow counter
    overflow : Option<u32>,
    application_description : Option<String>,
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

    receiver : channel::Receiver<Message>,
    sender : channel::Sender<Message>,
    mainloop_joinhandle : Option<JoinHandle<()>>,
}

impl DltUserInner {
    pub fn new() -> Result<Self,DltUserError> {
        //let ecu_id = [ecu_id[0] as u8,ecu_id[1] as u8, ecu_id[2] as u8, ecu_id[3] as u8];
        //let app_id = [app_id[0] as u8,app_id[1] as u8, app_id[2] as u8, app_id[3] as u8];
        let (sender,receiver) = channel::bounded::<Message>(100);
        let dlt_user = DltUserInner {
            ecu_id: None,
            app_id: None,
            logging_to_file: false,
            overflow: None,
            application_description: None,
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
            sender,
            receiver,
            mainloop_joinhandle :None,
        };

        Ok(dlt_user)
    }

    fn new_context(&self, context_id: String, description : String) -> Option<Context> {
        // TODO: Check defaults that may be set via configuration or environment 
        // variables
        Some(Context { 
            context_id, 
            log_level: 0, 
            trace_status: 1, 
            message_counter: 0, 
            description, 
            sender : self.sender.clone() })
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

pub struct Context {
    context_id : String,
    log_level : i8,
    trace_status : i8,
    message_counter : u8,
    description : String,
    sender : Sender<Message>,
}

struct MessageContext {
    message : Message,
}

impl MessageContext {
    pub fn new(ecu_id:String, verbose : bool) -> Result<Self,DltUserError> {

        let conf = MessageConfig {
            version: 1,
            counter: 0,
            endianness: dlt_core::dlt::Endianness::Big,
            ecu_id : None,
            session_id: None,
            timestamp: None,
            payload: if verbose {
                PayloadContent::Verbose(Vec::new())
            } else {
                PayloadContent::NonVerbose(0, Vec::new())
            },
            extended_header_info: None,
        };

        let storage_header = StorageHeader {
            timestamp: DltTimeStamp::from_ms(0),
            ecu_id,
        };
        let message = Message::new(conf, Some(storage_header));
        Ok(MessageContext { message })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.message.as_bytes()
    }


}


#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn basic() {
       let dlt_user = dlt_user();
       dlt_user.set_ecu_id("ECU1".to_string());
       dlt_user.set_app_info("APP1".to_string(), "Application description".to_string());
       // This sleep is to prevent the test from exiting before the mainloop task has
       // started
       std::thread::sleep(Duration::from_secs(2));
    }
}
