use crate::DltUserInner;
use dlt_core::dlt::Error;

use libc::{self, c_void, mkdir};
use std::ffi::CString;
use std::fs::File;

use std::io::{Read, Write};

use std::fs;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::prelude::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process;
use std::{thread, time};

static LOG_PATH: &str = "/tmp/dlt";
static DIR: &str = "/tmp/dltpipes";
static USER_PATH: &str = "/tmp/dltpipes/dlt";
static CONFIG: &str = "../libdlt/testdata/daemon.conf";

static IREAD: u32 = 400; //0400 read by user
static IWRITE: u32 = 0200; //0200  write by user
static IRGRP: u32 = 0400 >> 3; //0400>>3 read by group
static IWGRP: u32 = 0200 >> 3; //0200>>3 write by group

pub(crate) fn incoming_fifo() -> Result<(File, PathBuf), Error> {
    let mut user_path = USER_PATH.to_string();
    let process_id = process::id().to_string();
    user_path.push_str(&process_id);

    let path = PathBuf::from(user_path.clone());

    let mode = 0o3777; //read write and execute for owner and for group

    //create tmp/dltpipes path with permissions
    let dirpath = Path::new(&DIR);
    if !dirpath.exists() {
        fs::create_dir(DIR)?;
    }

    let mut perms = fs::metadata(DIR)?.permissions();
    perms.set_mode(mode);
    fs::set_permissions(DIR, perms)?;

    unsafe { libc::unlink(user_path.as_ptr() as *const i8) };
    //fs::remove_file(&user_path)?;

    let filename = CString::new(user_path.clone()).unwrap();

    let fs = unsafe {
        libc::mkfifo(filename.as_ptr(), IREAD | IWRITE | IRGRP | IWGRP) // as_ptr moved here
    };

    //opening path handles
    let mut user_path_handle = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(user_path.clone())?;
    thread::sleep(time::Duration::from_secs(10));

    Ok((user_path_handle, path))
}

pub(crate) fn outgoing_fifo() -> Result<File, Error> {
    let mut log_path = LOG_PATH.to_string();

    let mut log_path_handle = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_WRONLY | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(log_path)?;

    Ok(log_path_handle)
}

mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn incoming_test() {
        let mut dltuserinner = DltUserInner::new(CONFIG).unwrap();
        let (incoming, handle) = incoming_fifo().unwrap();
        dltuserinner.dlt_user_handle = Some(incoming);
        dltuserinner.user_path = Some(handle);
        // println!("{:?}",dltuserinner.dlt_user_handle);
    }

    #[test]
    fn outgoing_test() {
        let mut dltuserinner = DltUserInner::new(CONFIG).unwrap();
        let incoming = outgoing_fifo().unwrap();
        dltuserinner.dlt_log_handle = Some(incoming);
        println!("{:?}", dltuserinner.dlt_log_handle);
    }

    #[test]
    fn writing_and_reading_from_incoming_test() {
        let mut dltuserinner = DltUserInner::new(CONFIG).unwrap();
        let (mut incoming, handle) = incoming_fifo().unwrap();

        incoming.write(b"hello world");
        thread::sleep(time::Duration::from_secs(3));

        let mut buffer = [0; 1024];

        let res = match incoming.read(&mut buffer) {
            Ok(bytes_read) => {
                let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("{}", message);
            }
            Err(e) => panic!("Error reading from pipe:{}", e),
        };
    }
}
