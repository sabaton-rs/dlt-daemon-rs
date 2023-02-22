use crate::{fifo::fifo_connection, mainloop::mainloop};
use async_std::channel::{self, Sender};
use dlt_core::dlt::{DltTimeStamp, Message, MessageConfig, PayloadContent, StorageHeader};
use libc::EILSEQ;
use libdlt::{
    config::DaemonConfig,
    error::{DltError, DltUserError},
};
use std::io::Error;
use std::{
    env,
    fs::File,
    io::{IoSlice, Write},
    mem::MaybeUninit,
    sync::{Arc, Mutex, Once},
    thread::JoinHandle,
    u8,
};
use user_header::{user_control_message::RegisterApplication, UserHeader, UserMessageType};

pub(crate) mod fifo;
pub(crate) mod log;
pub(crate) mod mainloop;
pub(crate) mod user_header;
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

pub(crate) fn dlt_user_log_send_register(
    inner: &mut DltUserInner,
    user_header: &UserHeader,
    register_application: &RegisterApplication,
) -> Result<(), Error> {
    /* writing into log file */

    let bufs = [
        IoSlice::new(any_as_u8_slice(user_header)),
        IoSlice::new(any_as_u8_slice(register_application)),
        IoSlice::new(inner.application_description.as_bytes()),
    ];
    let result =inner.dlt_log_handle.as_ref().unwrap().write_vectored(&bufs);
    match result {
        Ok(_) => Ok(()),
        Err(error) => Err(error),
    }
}
pub struct DltUser {
    inner: Arc<Mutex<DltUserInner>>,
}
static DLT_PACKAGE_MINOR_VERSION: u32 = 2;
static DLT_PACKAGE_MAJOR_VERSION: u32 = 18;

fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    }
}

impl DltUser {
    pub fn set_ecu_id(&self, ecu_id: String) {
        self.inner.lock().unwrap().ecu_id = Some(ecu_id);
    }
    pub fn set_app_info(&self, app_id: String, description: String) {
        self.inner.lock().unwrap().app_id = Some(app_id);
        self.inner.lock().unwrap().application_description = description;
    }

    // Create a new Context for logging
    pub(crate) fn new_context(&self, context_id: &str, description: &str) -> Option<Context> {
        self.inner
            .lock()
            .unwrap()
            .new_context(context_id, description)
    }

    pub fn dlt_check_library_version(&self, minor_version: u32, major_version: u32) {
        if major_version != DLT_PACKAGE_MINOR_VERSION || minor_version != DLT_PACKAGE_MAJOR_VERSION
        {
            println!("Unsupported version of DLT Library");
        } else {
            println!("DLT Library version check successful");
            println!("found version {:?}.{:?}", major_version, minor_version);
        }
    }

    pub fn dlt_register_app(&self, app_id: &str, description: &str) -> Result<(), Error> {
        let dlt_package_minor_version = 18;
        let dlt_package_major_version = 2;

        self.dlt_check_library_version(dlt_package_minor_version, dlt_package_major_version);

        self.set_app_info(app_id.to_owned(), description.to_owned());

        if let Ok(mut inner) = self.inner.lock() {
            let user_header = UserHeader::new(UserMessageType::RegisterApplication);
            let register_application = RegisterApplication::new(&inner);

            let res = fifo_connection(&mut inner);

            let ret = dlt_user_log_send_register(&mut inner, &user_header, &register_application);
        }

        Ok(())
    }

    pub fn register_context(&self, context_id: &str, description: &str) -> Result<Context,DltError>  {
        self.new_context(context_id, description).ok_or(DltError::DltReturnWrongParameter)
    }

}

pub fn dlt_env_extract_ll_set(
    env_ini_loglevels: &str,
    initial_log_levels: &mut Vec<InitialLogLevel>,
) {
    let value: Vec<&str> = env_ini_loglevels.split_terminator(':').collect();
    let app_id: u32 = value[0].parse().unwrap();
    let context_id: u32 = value[1].parse().unwrap();
    let ll: i8 = value[2].parse().unwrap();
    let initial_log_level = InitialLogLevel {
        app_id: app_id,
        context_id: context_id,
        log_level: ll,
    };
    initial_log_levels.push(initial_log_level);
    println!(
        "env values: app-id: {:?} cont.id: {:?} ll: {:?}",
        initial_log_levels[0].app_id,
        initial_log_levels[0].context_id,
        initial_log_levels[0].log_level,
    );
}



fn start_async_mainloop(dlt_user_inner: Arc<Mutex<DltUserInner>>) {
    let dlt_user_inner_copy = dlt_user_inner.clone();
    let main = std::thread::spawn(move || async_std::task::block_on(mainloop(dlt_user_inner_copy)));
    dlt_user_inner
        .lock()
        .unwrap()
        .mainloop_joinhandle
        .replace(main);
}

/// Function to retrieve singleton DltUser
pub fn dlt_user() -> &'static DltUser {
    static mut DLT_USER: MaybeUninit<DltUser> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            // Make it
            #[cfg(test)]
            let config_path = "../libdlt/testdata/daemon.conf";
            #[cfg(not(test))]
            let config_path = "/etc/daemon.conf";

            let dlt_user = DltUser {
                inner: Arc::new(Mutex::new(DltUserInner::new(config_path).unwrap())),
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
    ecu_id: Option<String>,
    app_id: Option<String>,
    config: DaemonConfig,
    // if we are logging to file
    dlt_log_handle: Option<File>,
    dlt_user_handle: Option<File>,
    logging_to_file: bool,
    // overflow counter
    overflow: Option<u32>,
    application_description: String,
    verbose_mode: bool,
    use_extended_header_for_non_verbose: bool,
    /**< Use extended header for non verbose: 1 enabled, 0 disabled */
    with_session_id: bool,
    /**< Send always session id: 1 enabled, 0 disabled */
    with_timestamp: bool,
    /**< Send always timestamp: 1 enabled, 0 disabled */
    with_ecu_id: bool,
    enable_local_print: bool,
    local_print_mode: LocalPrintMode,
    log_buf_len: u32,
    log_msg_buf_max_size: u32,
    log_state: LogState,
    contexts: Vec<ContextStore>,
    initial_log_levels: Vec<InitialLogLevel>,
    receiver: channel::Receiver<Message>,
    sender: channel::Sender<Message>,
    mainloop_joinhandle: Option<JoinHandle<()>>,
}

impl DltUserInner {
    pub fn new(config_path: &str) -> Result<Self, DltUserError> {
        //let ecu_id = [ecu_id[0] as u8,ecu_id[1] as u8, ecu_id[2] as u8, ecu_id[3] as u8];
        //let app_id = [app_id[0] as u8,app_id[1] as u8, app_id[2] as u8, app_id[3] as u8];
        let (sender, receiver) = channel::bounded::<Message>(100);
        let dlt_user = DltUserInner {
            ecu_id: None,
            app_id: None,
            config: DaemonConfig::from_file(config_path).unwrap(),
            dlt_log_handle: None,
            dlt_user_handle: None,
            logging_to_file: false,
            overflow: None,
            application_description: String::new(),
            verbose_mode: true,
            use_extended_header_for_non_verbose: true,
            with_session_id: true,
            with_timestamp: true,
            with_ecu_id: true,
            enable_local_print: false,
            local_print_mode: LocalPrintMode::Unset,
            log_state: LogState::default(),
            contexts: Vec::new(),
            initial_log_levels: Vec::new(),
            sender,
            receiver,
            mainloop_joinhandle: None,
            log_buf_len: 1390,           //maximum size of each user buffer
            log_msg_buf_max_size: 65535, //Maximum log msg size as per autosar standard
        };

        Ok(dlt_user)
    }

    fn new_context(&mut self, context_id: &str, description: &str) -> Option<Context> {
       
        if !context_id.is_ascii() {
            return None
        }

        if !description.is_ascii() {
            return None
        }

        let mut context_id_bytes = [0u8;4];
        
        for (i,value) in context_id.as_bytes().iter().enumerate().take(4) {
            context_id_bytes[i] = *value;
        }

        // TODO: Check defaults that may be set via configuration or environment
        // variables

        let inner = 
        
        ContextInner {
            context_id : context_id_bytes,
            log_level: 0,
            trace_status: 1,
            message_counter: 0,
            description : description.to_owned(),
            sender: self.sender.clone(),
        };
        let context_store = ContextStore{ inner: Arc::new(inner) };
        // bail out if this context is already created
        if self.contexts.contains(&context_store) {
            println!("This context already exists");
            return None
        }
        
        self.contexts.push(context_store.clone());
        Some(Context { store: context_store } )
    }
}

pub(crate) fn dltinitcommon(
    dltuserinner: &mut DltUserInner,
) -> Result<&mut DltUserInner, DltError> {
    match env::var("DLT_DISABLE_EXTENDED_HEADER_FOR_NONVERBOSE") {
        Ok(val) => {
            match val.parse::<i32>() {
                Ok(n) => {
                    dltuserinner.use_extended_header_for_non_verbose = match n {
                        1 => false,
                        0 => true,
                        _ => true,
                    };
                    println!(
                        "env diabled exteded-verbose: {:?}",
                        dltuserinner.use_extended_header_for_non_verbose
                    );
                }
                Err(_) => println!("parsing error"),
            };
        }
        Err(e) => {
            dltuserinner.use_extended_header_for_non_verbose = true;
            println!(
                "default use extended-header: {:?}",
                dltuserinner.use_extended_header_for_non_verbose
            )
        }
    };

    match env::var("DLT_LOCAL_PRINT_MODE") {
        Ok(val) => {
            dltuserinner.local_print_mode = match val.as_str() {
                "Automatic" => LocalPrintMode::Automatic,
                "ForceOn" => LocalPrintMode::ForceOn,
                "ForceOff" => LocalPrintMode::ForceOff,
                _ => LocalPrintMode::Unset,
            };
            println!("env local_print: {:?}", dltuserinner.local_print_mode);
        }
        Err(e) => {
            dltuserinner.local_print_mode = LocalPrintMode::Unset;
            println!(
                "default local-print-mode={:?}",
                dltuserinner.local_print_mode
            )
        }
    };

    match env::var("DLT_USER_ENV_BUFFER_MIN_SIZE") {
        Ok(val) => {
            match val.parse::<u32>() {
                Ok(min) => {
                    if min > dltuserinner.config.ring_buffer_min_size.try_into().unwrap() {
                        dltuserinner.config.ring_buffer_min_size = min;
                        println!(
                            "env min-size: {:?}",
                            dltuserinner.config.ring_buffer_min_size
                        );
                    }
                }
                Err(_) => println!("parsing error"),
            };
        }
        Err(e) => {
            dltuserinner.config.ring_buffer_min_size = 500000;
            //Err(e) => {
            println!(
                "default min: {:?}",
                dltuserinner.config.ring_buffer_min_size
            )
        }
    };

    match env::var("DLT_USER_ENV_BUFFER_MAX_SIZE") {
        Ok(val) => {
            match val.parse::<u32>() {
                Ok(max) => {
                    if max < dltuserinner.config.ring_buffer_max_size.try_into().unwrap() {
                        dltuserinner.config.ring_buffer_max_size = max;
                        println!(
                            "env max-size: {:?}",
                            dltuserinner.config.ring_buffer_max_size
                        );
                    }
                }
                Err(_) => println!("parsing error"),
            };
        }
        Err(e) => {
            dltuserinner.config.ring_buffer_max_size = 10000000;
            println!(
                "default max: {:?}",
                dltuserinner.config.ring_buffer_max_size
            )
        }
    };

    match env::var("DLT_USER_ENV_BUFFER_STEP_SIZE") {
        Ok(val) => {
            match val.parse::<u32>() {
                Ok(step) => {
                    if step
                        > dltuserinner
                            .config
                            .ring_buffer_step_size
                            .try_into()
                            .unwrap()
                    {
                        dltuserinner.config.ring_buffer_step_size = step;
                        println!(
                            "env buf-step: {:?}",
                            dltuserinner.config.ring_buffer_step_size
                        );
                    }
                }
                Err(_) => println!("parsing error"),
            };
        }
        Err(e) => {
            dltuserinner.config.ring_buffer_step_size = 500000;
            println!(
                "default step: {:?}",
                dltuserinner.config.ring_buffer_step_size
            )
        }
    };

    match env::var("DLT_LOG_MSG_BUF_LEN") {
        Ok(val) => {
            match val.parse::<u32>() {
                Ok(len) => {
                    if len > dltuserinner.log_msg_buf_max_size {
                        dltuserinner.log_buf_len = dltuserinner.log_msg_buf_max_size;
                        println!("env buf-len: {:?}", dltuserinner.log_buf_len);
                    } else {
                        dltuserinner.log_buf_len = len;
                        println!("env buf-len: {:?}", dltuserinner.log_buf_len);
                    }
                }
                Err(_) => println!("parsing error"),
            };
        }
        Err(e) => {
            dltuserinner.log_buf_len = 1390;
            println!("default len: {:?}", dltuserinner.log_buf_len)
        }
    };

    match env::var("DLT_INITIAL_LOG_LEVEL") {
        Ok(env_initial_log_levels) => {
            dlt_env_extract_ll_set(
                &env_initial_log_levels,
                &mut dltuserinner.initial_log_levels,
            );
        }
        Err(_) => println!("no loglevel info"),
    }

    match env::var("DLT_DISABLE_INJECTION_MSG_AT_USER") {
        Ok(_inj_mode) => {
            dltuserinner.config.injection_mode = false;
            println!("env dlt-inj: {:?}", dltuserinner.config.injection_mode)
        }
        Err(_) => {
            dltuserinner.config.injection_mode = true;
            println!("default inj: {:?}", dltuserinner.config.injection_mode)
        }
    }

    Ok(dltuserinner)
}

#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
pub struct InitialLogLevel {
    app_id: u32,
    context_id: u32,
    log_level: i8,
}

/// The opaque context structure that is seen by the user
pub struct Context {
    store : ContextStore,
}

#[derive(Clone,PartialEq)]
pub(crate) struct ContextStore {
    inner: Arc<ContextInner>,
}

#[derive(Clone)]
struct ContextInner {
    context_id: [u8;4],
    log_level: i8,
    trace_status: i8,
    message_counter: u8,
    description: String,
    sender: Sender<Message>,
}

impl PartialEq for ContextInner {
    fn eq(&self, other: &Self) -> bool {
        self.context_id == other.context_id
    }
}

enum DLT_LOG {
    DLT_LOG_TO_CONSOLE = 0,
    DLT_LOG_TO_SYSLOG = 1,
    DLT_LOG_TO_FILE = 2,
    DLT_LOG_TO_STDERR = 3,
    DLT_LOG_DROPPED = 4,
}

struct MessageContext {
    message: Message,
}

impl MessageContext {
    pub fn new(ecu_id: String, verbose: bool) -> Result<Self, DltUserError> {
        let conf = MessageConfig {
            version: 1,
            counter: 0,
            endianness: dlt_core::dlt::Endianness::Big,
            ecu_id: None,
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

    use libc::sleep;

    use super::*;

    const CONFIG: &str = "../libdlt/testdata/daemon.conf";

    #[test]
    fn basic() {
        let dlt_user = dlt_user();
        dlt_user.dlt_register_app("EXU1", "THIS IS FIRST TEST");

        // This sleep is to prevent the test from exiting before the mainloop task has
        // started
        std::thread::sleep(Duration::from_secs(20));
    }
    #[test]
    fn test_verbose() {
        let mut dltuserinner = DltUserInner::new(CONFIG).unwrap();
        let key = "DLT_DISABLE_EXTENDED_HEADER_FOR_NONVERBOSE";
        env::set_var(key, "1");
        let res = dltinitcommon(&mut dltuserinner);
        assert_eq!(res.unwrap().use_extended_header_for_non_verbose, false);
        env::remove_var(key);
    }

    #[test]
    fn log_print_mode() {
        let mut dltuserinner = DltUserInner::new(CONFIG).unwrap();
        let key = "DLT_LOCAL_PRINT_MODE";
        env::set_var(key, "ForceOn");
        let res = dltinitcommon(&mut dltuserinner);
        assert_eq!(res.unwrap().local_print_mode, LocalPrintMode::ForceOn);
        env::remove_var(key);
    }

    #[test]
    fn min_buf_size() {
        let mut dltuserinner = DltUserInner::new(CONFIG).unwrap();
        let key = "DLT_USER_ENV_BUFFER_MIN_SIZE";
        env::set_var(key, "600000");
        let res = dltinitcommon(&mut dltuserinner);
        assert_eq!(res.unwrap().config.ring_buffer_min_size, 600000);
        env::remove_var(key);
    }
    #[test]
    fn max_buf_size() {
        let mut dltuserinner = DltUserInner::new(CONFIG).unwrap();
        let key = "DLT_USER_ENV_BUFFER_MAX_SIZE";
        env::set_var(key, "650000");
        let res = dltinitcommon(&mut dltuserinner);
        assert_eq!(res.unwrap().config.ring_buffer_max_size, 650000);
        env::remove_var(key);
    }

    #[test]
    fn buf_step_size() {
        let mut dltuserinner = DltUserInner::new(CONFIG).unwrap();
        let key = "DLT_USER_ENV_BUFFER_STEP_SIZE";
        env::set_var(key, "700000");
        let res = dltinitcommon(&mut dltuserinner);
        assert_eq!(res.unwrap().config.ring_buffer_step_size, 700000);
        env::remove_var(key);
    }

    #[test]
    fn buf_len() {
        let mut dltuserinner = DltUserInner::new(CONFIG).unwrap();
        let key = "DLT_LOG_MSG_BUF_LEN";
        env::set_var(key, "65535");
        let res = dltinitcommon(&mut dltuserinner);
        assert_eq!(res.unwrap().log_buf_len, 65535);
        env::remove_var(key);
    }

    #[test]
    fn test_init_ll_set() {
        let mut dltuserinner = DltUserInner::new(CONFIG).unwrap();
        let key = "DLT_INITIAL_LOG_LEVEL";
        env::set_var(key, "1234:4567:3");
        let res = dltinitcommon(&mut dltuserinner);
        assert_eq!(
            res.unwrap().initial_log_levels[0],
            InitialLogLevel {
                app_id: 1234,
                context_id: 4567,
                log_level: 3
            }
        );
        env::remove_var(key);
    }

    #[test]
    fn test_env_inj_mode() {
        let mut dltuserinner = DltUserInner::new(CONFIG).unwrap();
        let key = "DLT_DISABLE_INJECTION_MSG_AT_USER";
        env::set_var(key, "1");
        let res = dltinitcommon(&mut dltuserinner);
        assert_eq!(res.unwrap().config.injection_mode, false);
        env::remove_var(key);
    }
}
