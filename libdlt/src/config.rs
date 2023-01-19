use std::{path::{PathBuf, Path}, time::Duration};

use ini::configparser::ini::Ini;

use crate::error::DltError;
//LOG_EMERG = 0, LOG_ALERT = 1, LOG_CRIT = 2, LOG_ERR = 3, LOG_WARNING = 4, LOG_NOTICE = 5, LOG_INFO = 6, LOG_DEBUG = 7
pub enum LogLevel {
    Emergency=0,
    Alert=1,
    Critical=2,
    Error=3,
    Warning=4,
    Notice=5,
    Info=6,
    Debug=7,
}

pub enum DaemonLoggingMode {
    Stdout,
    Syslog,
    StdError,
    File(PathBuf),
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
    pub logging_level : LogLevel,
    pub timeout_on_send : Duration,
    pub ring_buffer_min_size : u32,
    pub ring_buffer_max_size : u32,
    pub ring_buffer_step_size : u32,
    pub daemon_fifo_size: u32,
    
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
                            match (k.as_str(),v) {
                                ("verbose",Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    println!("Value:{val}");
                                    if val > 0 {
                                        conf.verbose = true;
                                    } 
                                }
                                ("daemonize",Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    println!("Daemon:{val}");
                                    if val > 0 {
                                        conf.daemonize = true;
                                    }
                                }
                                ("send_serial_header",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    println!("serial: {val}");
                                    if val > 0 {
                                        conf.send_serial_header = true;
                                    }
                                }
                                ("send_context_registration",Some(value)) =>{
                                    let val: u32 = value.parse().unwrap();
                                    println!("context_Reg: {val}");
                                    if val > 0 {
                                        conf.send_context_registration = true;
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
        assert!(config.daemonize,"{}", 1);
        assert!(config.send_serial_header,"{}", true);
        assert!(config.send_context_registration);
        
    }
}