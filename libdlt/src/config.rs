use std::{path::{PathBuf, Path}, time::Duration};

use ini::configparser::ini::Ini;

use crate::error::DltError;
//LOG_EMERG = 0, LOG_ALERT = 1, LOG_CRIT = 2, LOG_ERR = 3, LOG_WARNING = 4, LOG_NOTICE = 5, LOG_INFO = 6, LOG_DEBUG = 7

#[derive(Debug,PartialEq)]
pub enum LogLevel {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Debug,
}
#[derive(Debug,PartialEq)]
pub enum DaemonLoggingMode {
    Stdout,
    Syslog,
    StdError,
    File(PathBuf),
}
#[derive(Debug,PartialEq)]
pub enum DltLogLevel {
    DltLogOff,
    DltLogFatal,
    DltLogError,
    DltLogWarn,
    DltLogInfo,
    DltLogDebug,
    DltLogVerbose,
}
#[derive(Debug,PartialEq)]
pub enum OfflineTraceFileName {
    TimeStampBased,
    IndexBased,
}

pub struct DaemonConfig {
    pub verbose : bool,
    pub daemonize : bool,
    pub send_serial_header : bool,
    pub send_context_registration : bool,
    pub send_context_registration_option : u8, 
    pub send_message_time : bool,
    pub ecu_id : String,
    pub shared_memory_size : u32,
    pub persistance_storage_path : PathBuf,
    pub logging_mode : DaemonLoggingMode,
    //LoggingFilename
    pub logging_level : LogLevel,
    pub timeout_on_send : Duration,
    pub ring_buffer_min_size : u32,
    pub ring_buffer_max_size : u32,
    pub ring_buffer_step_size : u32,
    pub daemon_fifo_size: u32,
    pub context_log_level: DltLogLevel,
    pub context_trace_status: bool,
    pub force_context_loglevel_and_tracestatus: bool,
    pub injection_mode: bool,
 
    // TODO: Other config fields
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            daemonize: false,
            send_serial_header: false,
            send_context_registration: true,
            send_context_registration_option: 7,
            send_message_time: true,
            ecu_id: String::from("ECU1"),
            shared_memory_size: 100000,
            persistance_storage_path: PathBuf::from("/tmp"),
            logging_mode: DaemonLoggingMode::Stdout,
            logging_level: LogLevel::Error,
            timeout_on_send: Duration::from_secs(4),
            ring_buffer_min_size: 500000,
            ring_buffer_max_size: 10000000,
            ring_buffer_step_size: 500000,
            daemon_fifo_size: 65536,
            context_log_level: DltLogLevel::DltLogInfo,
            context_trace_status: false,
            force_context_loglevel_and_tracestatus: false,
            injection_mode: true,

        }
    } 

}

impl DaemonConfig {

    pub fn from_file(conf : &str) -> Result<Self, DltError> {
    
        let mut daemon_conf = Ini::new();
        daemon_conf.load(conf).map_err(|e|DltError::ConfigFileError(e))?;

        let mut conf = DaemonConfig::default();

        if let Some(map) = daemon_conf.get_map() {
            for (section,map) in map.iter() {
                println!("Section:{section}");
                match section.as_str() {
                    "default" => {  // Default section
                        for (k,v) in map.iter() {
                            // println!("k:{},v:{:?}",k,v);
                            match (k.as_str(),v) {
                                ("verbose",Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    //println!("Value:{val}");
                                    if val == 0 {
                                        conf.verbose = false;
                                    }
                                    else {
                                        conf.verbose = true;
                                    } 
                                }
                                ("daemonize",Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    // println!("Daemon:{val}");
                                    if val == 0 {
                                        conf.daemonize = false;
                                    }
                                    else {
                                        conf.daemonize = true;
                                    }
                                }
                                ("sendserialheader",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    // println!("serial: {val}");
                                    if val == 0 {
                                        conf.send_serial_header = false;
                                    }
                                    else {
                                        conf.send_serial_header = true;
                                    }
                                }
                                ("sendcontextregistration",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    // println!("context_Reg: {val}");
                                    if val == 0 {
                                        conf.send_context_registration = false;
                                    }
                                    else {
                                        conf.send_context_registration = true;
                                    }
                                }
                                ("sendcontextregistrationoption",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    // println!("con_reg_opt: {val}");
                                    if val > 0 {
                                        conf.send_context_registration_option = 7;
                                    }
                                }
                                ("sendmessagetime",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    // println!("message time: {val}");
                                    if val == 0 {
                                        conf.send_message_time= false;
                                    }
                                    else {
                                        conf.send_message_time =true;
                                    }
                                }
                                ("ecuid",Some(value)) =>{
                                    let val: String = value.parse().unwrap();
                                    // println!("ecu_id: {val}");
                                    if val.len() == 4 {
                                        conf.ecu_id= String::from(val);
                                    }
                                }
                                ("sharedmemorysize",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    // println!("shared_memory_value: {val}");
                                    if val > 0 {
                                        conf.shared_memory_size = val;
                                    }
                                }
                                ("persistancestoragepath",Some(value)) =>{
                                    let val: PathBuf = value.parse().unwrap();
                                    //println!("path: {val}");
                                        conf.persistance_storage_path = val;
                                    
                                }
                                ("loggingmode",Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    //println!("logmode: {val}");
                                    match val {
                                            0 => conf.logging_mode = DaemonLoggingMode::Stdout,
                                            1 => conf.logging_mode = DaemonLoggingMode::Syslog,
                                            2 => conf.logging_mode = DaemonLoggingMode::StdError,
                                            3 => conf.logging_mode = DaemonLoggingMode::File(PathBuf::from("/tmp/dlt.log")),
                                            _=>  conf.logging_mode = DaemonLoggingMode::Stdout,
                                    };
                                }
                                ("logginglevel",Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    //println!("log_level: {val}");
                                    match val {
                                            0 => conf.logging_level = LogLevel::Emergency,
                                            1 => conf.logging_level = LogLevel::Alert,
                                            2 => conf.logging_level = LogLevel::Critical,
                                            3 => conf.logging_level = LogLevel::Error,
                                            4 => conf.logging_level = LogLevel::Warning,
                                            5 => conf.logging_level = LogLevel::Notice,
                                            6 => conf.logging_level = LogLevel::Info,
                                            7 => conf.logging_level = LogLevel::Debug,
                                            _=>  conf.logging_level = LogLevel::Info,
                                    };
                                }
                                ("timeoutonsend",Some(value)) =>{
                                    let val: u64 = value.parse().unwrap();
                                    //println!("timeout_on_send: {val}");
                                    if val > 0 {
                                        conf.timeout_on_send = Duration::from_secs(val);
                                    }
                                }
                                ("ringbufferminsize",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    //println!("ring_buffer_min_size: {val}");
                                    if val > 0 {
                                        conf.ring_buffer_min_size = val;
                                    }
                                }
                                ("ringbuffermaxsize",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    //println!("ring_buffer_max_size: {val}");
                                    if val > 0 {
                                        conf.ring_buffer_max_size = val;
                                    }
                                }
                                ("ringbufferstepsize",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    //println!("ring_buffer_step_size: {val}");
                                    if val > 0 {
                                        conf.ring_buffer_step_size = val;
                                    }
                                }
                                ("daemonfifosize",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    // println!("daemon_fifo_size: {val}");  
                                    if val > 0 {
                                        conf.daemon_fifo_size = val;
                                    }
                                }
                                ("contextloglevel",Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    //println!("dltlog: {val}");
                                    match val {
                                            0 => conf.context_log_level = DltLogLevel::DltLogOff,
                                            1 => conf.context_log_level = DltLogLevel::DltLogFatal,
                                            2 => conf.context_log_level = DltLogLevel::DltLogError,
                                            3 => conf.context_log_level = DltLogLevel::DltLogWarn,
                                            4 => conf.context_log_level = DltLogLevel::DltLogInfo,
                                            5 => conf.context_log_level = DltLogLevel::DltLogDebug,
                                            6 => conf.context_log_level = DltLogLevel::DltLogVerbose,
                                            _=>  conf.context_log_level = DltLogLevel::DltLogInfo,
                                    };
                                }
                                ("contexttracestatus",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    // println!("context_trace_status: {val}");  
                                    if val == 0 {
                                        conf.context_trace_status = false;
                                    }
                                    else {
                                        conf.context_trace_status = true;
                                    }   
                                }
                                ("forcecontextloglevelandtracestatus",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    //println!("force_context_loglevel_and_tracestatus: {val}");  
                                    if val == 0 {
                                        conf.force_context_loglevel_and_tracestatus = false;
                                    }
                                    else {
                                        conf.force_context_loglevel_and_tracestatus = true;
                                    }   
                                }
                                ("injectionmode",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    // println!("injection_mode: {val}");  
                                    if val == 0 {
                                        conf.injection_mode = false;
                                    }
                                    else {
                                        conf.injection_mode = true;
                                    }   
                                }
                                

                                //TODO: implement remaining configs
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }    
        }

        Ok(conf)

    }
}


#[cfg(test)]
mod tests {
    use super::*;
 
    #[test]
    fn basic() {
        let config = DaemonConfig::from_file("/home/devuser/dlt/dlt-daemon-rs/libdlt/testdata/daemon.conf").unwrap();
        assert!(config.verbose);
        assert!(config.daemonize);
        assert!(config.send_serial_header);
        assert!(config.send_context_registration);
        assert_eq!(config.send_context_registration_option,7);
        assert_eq!(config.send_message_time,false);
        assert_eq!(config.ecu_id,"ECU1");
        assert_eq!(config.shared_memory_size,100000);
        assert_eq!(config.persistance_storage_path, PathBuf::from("/tmp"));
        assert_eq!(config.logging_level,LogLevel::Info);
        assert_eq!(config.logging_mode,DaemonLoggingMode::Stdout);
        assert_eq!(config.timeout_on_send,Duration::from_secs(4));
        assert_eq!(config.ring_buffer_min_size,500000);
        assert_eq!(config.ring_buffer_max_size,10000000);
        assert_eq!(config.ring_buffer_step_size,500000);
        assert_eq!(config.daemon_fifo_size,65536);
        assert_eq!(config.context_log_level,DltLogLevel::DltLogInfo);
        assert_eq!(config.context_trace_status,false);
        assert!(config.force_context_loglevel_and_tracestatus);
        assert!(config.injection_mode);


    }
}