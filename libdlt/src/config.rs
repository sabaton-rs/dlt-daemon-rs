use std::{net::IpAddr, path::PathBuf, str::FromStr, time::Duration};

use ini::configparser::ini::Ini;

use crate::error::DltError;
//LOG_EMERG = 0, LOG_ALERT = 1, LOG_CRIT = 2, LOG_ERR = 3, LOG_WARNING = 4, LOG_NOTICE = 5, LOG_INFO = 6, LOG_DEBUG = 7
#[derive(Debug, PartialEq)]
pub enum SendContextRegistrationOption {
    Apid = 3,
    Loglevel = 4,
    Tracestatus = 5,
    Ll = 6,
    Description = 7,
}
impl SendContextRegistrationOption {
    pub const CTID: SendContextRegistrationOption = SendContextRegistrationOption::Apid;
    pub const TS: SendContextRegistrationOption = SendContextRegistrationOption::Ll;
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum DaemonLoggingMode {
    Stdout,
    Syslog,
    StdError,
    File(PathBuf),
}
#[derive(Debug, PartialEq)]
pub enum DltLogLevel {
    DltLogOff,
    DltLogFatal,
    DltLogError,
    DltLogWarn,
    DltLogInfo,
    DltLogDebug,
    DltLogVerbose,
}
#[derive(Debug, PartialEq)]
pub enum OfflineTraceFileName {
    OfflineTraceFileNameIndexBased,
    OfflineTraceFileNameTimeStampBased,
}

pub struct DaemonConfig {
    pub verbose: bool,
    pub daemonize: bool,
    pub send_serial_header: bool,
    pub send_context_registration: bool,
    pub send_context_registration_option: SendContextRegistrationOption,
    pub send_message_time: bool,
    pub ecu_id: String,
    pub shared_memory_size: u32,
    pub persistance_storage_path: PathBuf,
    pub logging_mode: DaemonLoggingMode,
    pub logging_level: LogLevel,
    pub logging_filename: PathBuf,
    pub timeout_on_send: Duration,
    pub ring_buffer_min_size: u32,
    pub ring_buffer_max_size: u32,
    pub ring_buffer_step_size: u32,
    pub daemon_fifo_size: u32,
    pub context_log_level: DltLogLevel,
    pub context_trace_status: bool,
    pub force_context_loglevel_and_tracestatus: bool,
    pub injection_mode: bool,
    //Gateway
    pub gateway_mode: bool,
    pub gateway_config_file: PathBuf,
    //permission
    pub daemon_fifo_group: String,
    pub control_socket_path: PathBuf,
    pub offline_trace_directory: Option<PathBuf>,
    pub offline_trace_file_size: u32,
    pub offline_trace_max_size: u32,
    pub offline_trace_file_name: OfflineTraceFileName,
    pub print_ascii: bool,
    pub print_hex: bool,
    pub print_headers_only: bool,
    pub serial_port: Option<String>,
    pub rs232_baudrate: u32,
    pub rs232_sync_serial_header: bool,
    pub tcpsync_serial_header: bool,
    pub send_ecusoftware_version: u32,
    pub path_to_ecusoftware_version: Option<PathBuf>,
    pub send_timezone: u32,
    pub offline_logstorage_max_devices: bool,
    pub offline_logstorage_dir_path: Option<PathBuf>,
    pub offline_logstorage_timestamp: bool,
    pub offline_logstorage_delimiter: String,
    pub offline_logstorage_max_counter: u32,
    pub offline_logstorage_cache_size: u32,
    pub udpconnection_setup: bool,
    pub udpmulticast_ipaddress: IpAddr,
    pub udpmulticast_ipport: u32,
    pub bind_address: IpAddr,
    // TODO: Other config fields
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            daemonize: false,
            send_serial_header: false,
            send_context_registration: true,
            send_context_registration_option: SendContextRegistrationOption::Description,
            send_message_time: true,
            ecu_id: String::from("ECU1"),
            shared_memory_size: 100000,
            persistance_storage_path: PathBuf::from("/tmp"),
            logging_mode: DaemonLoggingMode::Stdout,
            logging_level: LogLevel::Error,
            logging_filename: PathBuf::from("/tmp/dlt.log"),
            timeout_on_send: Duration::from_secs(4),
            ring_buffer_min_size: 500000,
            ring_buffer_max_size: 10000000,
            ring_buffer_step_size: 500000,
            daemon_fifo_size: 65536,
            context_log_level: DltLogLevel::DltLogInfo,
            context_trace_status: false,
            force_context_loglevel_and_tracestatus: false,
            injection_mode: true,
            gateway_mode: false,
            gateway_config_file: PathBuf::from("/etc/dlt_gateway.conf"),
            daemon_fifo_group: String::from("/tmp/dlt"),
            control_socket_path: PathBuf::from("/tmp/dlt-ctrl.sock"),
            offline_trace_directory: None,
            offline_trace_file_size: 1000000,
            offline_trace_max_size: 4000000,
            offline_trace_file_name: OfflineTraceFileName::OfflineTraceFileNameTimeStampBased,
            print_ascii: true,
            print_hex: true,
            print_headers_only: true,
            serial_port: None,
            rs232_baudrate: 115200,
            rs232_sync_serial_header: true,
            tcpsync_serial_header: true,
            send_ecusoftware_version: 0,
            path_to_ecusoftware_version: None,
            send_timezone: 0,
            offline_logstorage_max_devices: false,
            offline_logstorage_dir_path: None,
            offline_logstorage_timestamp: true,
            offline_logstorage_delimiter: String::from("_"),
            offline_logstorage_max_counter: u32::max_value(),
            offline_logstorage_cache_size: 30000,
            udpconnection_setup: true,
            udpmulticast_ipaddress: IpAddr::from_str("225.0.0.37").unwrap(),
            udpmulticast_ipport: 3491,
            bind_address: IpAddr::from_str("0.0.0.0").unwrap(),
        }
    }
}

impl DaemonConfig {
    pub fn from_file(conf: &str) -> Result<Self, DltError> {
        let mut daemon_conf = Ini::new();
        daemon_conf.load(conf).map_err(DltError::ConfigFileError)?;

        let mut conf = DaemonConfig::default();

        if let Some(map) = daemon_conf.get_map() {
            for (section, map) in map.iter() {
                println!("Section:{section}");
                match section.as_str() {
                    "default" => {
                        // Default section
                        for (k, v) in map.iter() {
                            match (k.as_str(), v) {
                                ("verbose", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.verbose = false;
                                    } else {
                                        conf.verbose = true;
                                    }
                                }
                                ("daemonize", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.daemonize = false;
                                    } else {
                                        conf.daemonize = true;
                                    }
                                }
                                ("sendserialheader", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.send_serial_header = false;
                                    } else {
                                        conf.send_serial_header = true;
                                    }
                                }
                                ("sendcontextregistration", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.send_context_registration = false;
                                    } else {
                                        conf.send_context_registration = true;
                                    }
                                }
                                ("sendcontextregistrationoption", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.send_context_registration_option = match val {
                                        3 => SendContextRegistrationOption::Apid,
                                        4 => SendContextRegistrationOption::Loglevel,
                                        5 => SendContextRegistrationOption::Tracestatus,
                                        6 => SendContextRegistrationOption::TS,
                                        7 => SendContextRegistrationOption::Description,
                                        _ => SendContextRegistrationOption::Description,
                                    }
                                }
                                ("sendmessagetime", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.send_message_time = false;
                                    } else {
                                        conf.send_message_time = true;
                                    }
                                }
                                ("ecuid", Some(value)) => {
                                    let val: String = value.parse().unwrap();
                                    if val.is_empty() {
                                        conf.ecu_id = String::from("ECU1");
                                    } else if !val.is_empty() && val.len() <= 4 {
                                        conf.ecu_id = val;
                                    } else if val.len() > 4 {
                                        conf.ecu_id = val[0..4].to_owned();
                                    }
                                }
                                ("sharedmemorysize", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.shared_memory_size = val;
                                }
                                ("persistancestoragepath", Some(value)) => {
                                    let val: PathBuf = value.parse().unwrap();
                                    conf.persistance_storage_path = val;
                                }
                                ("loggingmode", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.logging_mode = match val {
                                        0 => DaemonLoggingMode::Stdout,
                                        1 => DaemonLoggingMode::Syslog,
                                        2 => DaemonLoggingMode::StdError,
                                        3 => DaemonLoggingMode::File(PathBuf::from("/tmp/dlt.log")),
                                        _ => DaemonLoggingMode::Stdout,
                                    };
                                }
                                ("logginglevel", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.logging_level = match val {
                                        0 => LogLevel::Emergency,
                                        1 => LogLevel::Alert,
                                        2 => LogLevel::Critical,
                                        3 => LogLevel::Error,
                                        4 => LogLevel::Warning,
                                        5 => LogLevel::Notice,
                                        6 => LogLevel::Info,
                                        7 => LogLevel::Debug,
                                        _ => LogLevel::Info,
                                    };
                                }
                                ("loggingfilename", Some(value)) => {
                                    let val: PathBuf = value.parse().unwrap();
                                    conf.logging_filename = val;
                                }
                                ("timeoutonsend", Some(value)) => {
                                    let val: u64 = value.parse().unwrap();
                                    conf.timeout_on_send = Duration::from_secs(val);
                                }
                                ("ringbufferminsize", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.ring_buffer_min_size = val;
                                }
                                ("ringbuffermaxsize", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.ring_buffer_max_size = val;
                                }
                                ("ringbufferstepsize", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.ring_buffer_step_size = val;
                                }
                                ("daemonfifosize", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.daemon_fifo_size = val;
                                }
                                ("contextloglevel", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.context_log_level = match val {
                                        0 => DltLogLevel::DltLogOff,
                                        1 => DltLogLevel::DltLogFatal,
                                        2 => DltLogLevel::DltLogError,
                                        3 => DltLogLevel::DltLogWarn,
                                        4 => DltLogLevel::DltLogInfo,
                                        5 => DltLogLevel::DltLogDebug,
                                        6 => DltLogLevel::DltLogVerbose,
                                        _ => DltLogLevel::DltLogInfo,
                                    };
                                }
                                ("contexttracestatus", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.context_trace_status = false;
                                    } else {
                                        conf.context_trace_status = true;
                                    }
                                }
                                ("forcecontextloglevelandtracestatus", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.force_context_loglevel_and_tracestatus = false;
                                    } else {
                                        conf.force_context_loglevel_and_tracestatus = true;
                                    }
                                }
                                ("injectionmode", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.injection_mode = false;
                                    } else {
                                        conf.injection_mode = true;
                                    }
                                }
                                ("gatewaymode", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.gateway_mode = false;
                                    } else {
                                        conf.gateway_mode = true;
                                    }
                                }
                                ("gatewayconfigfile", Some(value)) => {
                                    let val: PathBuf = value.parse().unwrap();
                                    conf.gateway_config_file = val;
                                }
                                ("daemonfifogroup", Some(value)) => {
                                    let val: String = value.parse().unwrap();
                                    conf.daemon_fifo_group = val;
                                }
                                ("controlsocketpath", Some(value)) => {
                                    let val: PathBuf = value.parse().unwrap();
                                    conf.control_socket_path = val;
                                }
                                ("offlinetracedirectory", Some(value)) => {
                                    if value.is_empty() {
                                        conf.offline_trace_directory = None;
                                    } else {
                                        conf.offline_trace_directory = Some(PathBuf::from(value));
                                    }
                                }
                                ("offlinetracefilesize", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.offline_trace_file_size = val;
                                }
                                ("offlinetracemaxsize", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.offline_trace_max_size = val;
                                }
                                ("offlinetracefilenametimestampbased", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.offline_trace_file_name = match val {
                                        0 => OfflineTraceFileName::OfflineTraceFileNameIndexBased,
                                        1 => {
                                            OfflineTraceFileName::OfflineTraceFileNameTimeStampBased
                                        }
                                        _ => {
                                            OfflineTraceFileName::OfflineTraceFileNameTimeStampBased
                                        }
                                    };
                                }
                                ("printascii", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.print_ascii = false;
                                    } else {
                                        conf.print_ascii = true;
                                    }
                                }
                                ("printhex", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.print_hex = false;
                                    } else {
                                        conf.print_hex = true;
                                    }
                                }
                                ("printheadersonly", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.print_headers_only = false;
                                    } else {
                                        conf.print_headers_only = true;
                                    }
                                }
                                ("rs232devicename", Some(value)) => {
                                    if value.is_empty() {
                                        conf.serial_port = None;
                                    } else {
                                        conf.serial_port = Some(String::from(value));
                                    }
                                }
                                ("rs232baudrate", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.rs232_baudrate = val;
                                }
                                ("rs232syncserialheader", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.rs232_sync_serial_header = false;
                                    } else {
                                        conf.rs232_sync_serial_header = true;
                                    }
                                }
                                ("tcpsyncserialheader", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.tcpsync_serial_header = val != 0
                                }
                                ("sendecusoftwareversion", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.send_ecusoftware_version = val;
                                }
                                ("pathtoecusoftwareversion", Some(value)) => {
                                    if value.is_empty() {
                                        conf.path_to_ecusoftware_version = None;
                                    } else {
                                        conf.path_to_ecusoftware_version =
                                            Some(PathBuf::from(value));
                                    }
                                }
                                ("sendtimezone", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.send_timezone = val;
                                }
                                ("offlinelogstoragemaxdevices", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.offline_logstorage_max_devices = false;
                                    } else {
                                        conf.offline_logstorage_max_devices = true;
                                    }
                                }
                                ("offlinelogstoragedirpath", Some(value)) => {
                                    if value.is_empty() {
                                        conf.offline_logstorage_dir_path = None;
                                    } else {
                                        conf.offline_logstorage_dir_path =
                                            Some(PathBuf::from(value));
                                    }
                                }
                                ("offlinelogstoragetimestamp", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.offline_logstorage_timestamp = false;
                                    } else {
                                        conf.offline_logstorage_timestamp = true;
                                    }
                                }
                                ("offlinelogstoragedelimiter", Some(value)) => {
                                    let val: String = value.parse().unwrap();
                                    if !val.contains('_') {
                                        conf.offline_logstorage_delimiter = val;
                                    }
                                }
                                ("offlinelogstoragemaxcounter", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.offline_logstorage_max_counter = val;
                                }
                                ("offlinelogstoragecachesize", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.offline_logstorage_cache_size = val;
                                }
                                ("udpconnectionsetup", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    if val == 0 {
                                        conf.udpconnection_setup = false;
                                    } else {
                                        conf.udpconnection_setup = true;
                                    }
                                }
                                ("udpmulticastipaddress", Some(value)) => {
                                    let val: String = value.parse().unwrap();
                                    if let Ok(ip) = IpAddr::from_str(&val) {
                                        conf.udpmulticast_ipaddress = ip;
                                    }
                                }
                                ("udpmulticastipport", Some(value)) => {
                                    let val: u32 = value.parse().unwrap();
                                    conf.udpmulticast_ipport = val;
                                }
                                ("bindaddress", Some(value)) => {
                                    let val: String = value.parse().unwrap();
                                    if let Ok(ip) = IpAddr::from_str(&val) {
                                        conf.bind_address = ip;
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
        let config =
            DaemonConfig::from_file("/home/devuser/dlt/dlt-daemon-rs/libdlt/testdata/daemon.conf")
                .unwrap();
        assert!(config.verbose);
        assert!(config.daemonize);
        assert!(config.send_serial_header);
        assert!(config.send_context_registration);
        assert_eq!(
            config.send_context_registration_option,
            SendContextRegistrationOption::Description
        );
        assert_eq!(config.send_message_time, false);
        assert_eq!(config.ecu_id, "ECU1");
        assert_eq!(config.shared_memory_size, 100000);
        assert_eq!(config.persistance_storage_path, PathBuf::from("/tmp"));
        assert_eq!(config.logging_level, LogLevel::Info);
        assert_eq!(config.logging_mode, DaemonLoggingMode::Stdout);
        assert_eq!(config.logging_filename, PathBuf::from("/tmp/dlt.log"));
        assert_eq!(config.timeout_on_send, Duration::from_secs(4));
        assert_eq!(config.ring_buffer_min_size, 500000);
        assert_eq!(config.ring_buffer_max_size, 10000000);
        assert_eq!(config.ring_buffer_step_size, 500000);
        assert_eq!(config.daemon_fifo_size, 65536);
        assert_eq!(config.context_log_level, DltLogLevel::DltLogInfo);
        assert_eq!(config.context_trace_status, false);
        assert!(config.force_context_loglevel_and_tracestatus);
        assert!(config.injection_mode);
        assert!(config.gateway_mode);
        assert_eq!(
            config.gateway_config_file,
            PathBuf::from("/etc/dlt_gateway.conf")
        );
        assert_eq!(config.daemon_fifo_group,String::from("dlt_user_apps_group"));
        assert_eq!(
            config.control_socket_path,
            PathBuf::from("/tmp/dlt-ctrl.sock")
        );
        assert_eq!(config.offline_trace_directory, Some(PathBuf::from("/tmp")));
        assert_eq!(config.offline_trace_file_size, 1000000);
        assert_eq!(config.offline_trace_max_size, 4000000);
        assert_eq!(
            config.offline_trace_file_name,
            OfflineTraceFileName::OfflineTraceFileNameTimeStampBased
        );
        assert!(config.print_ascii);
        assert!(config.print_hex);
        assert!(config.print_headers_only);
        assert_eq!(config.serial_port, Some(String::from("/dev/ttyS0")));
        assert_eq!(config.rs232_baudrate, 115200);
        assert!(config.rs232_sync_serial_header);
        assert!(config.tcpsync_serial_header);
        assert_eq!(config.send_ecusoftware_version, 0);
        assert_eq!(config.send_timezone, 0);
        assert!(config.offline_logstorage_max_devices);
        assert_eq!(config.offline_logstorage_timestamp, false);
        assert_eq!(config.offline_logstorage_delimiter, "_");
        assert_eq!(config.offline_logstorage_max_counter, 999);
        assert_eq!(config.offline_logstorage_cache_size, 30000);
        assert!(config.udpconnection_setup);
        assert_eq!("225.0.0.37".parse(), Ok(config.udpmulticast_ipaddress));
        assert_eq!("160.48.199.97".parse(), Ok(config.bind_address));
        assert_eq!(config.udpmulticast_ipport, 3491);
    }
}
