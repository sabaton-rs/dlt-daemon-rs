use crate::DltUserInner;
use dlt_core::dlt::Error;
use libc::{self, c_void, mkdir};
use std::ffi::CString;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::fd::{FromRawFd, IntoRawFd};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::prelude::PermissionsExt;
use std::process;
use std::{fs, result};
use std::{thread, time};

static LOG_PATH: &str = "/tmp/dlt";
static DIR: &str = "/tmp/dltpipes";
static USER_PATH: &str = "/tmp/dltpipes/dlt";
static CONFIG: &str = "../libdlt/testdata/daemon.conf";

static IREAD: u32 = 400; //0400 read by user
static IWRITE: u32 = 0200; //0200  write by user
static IRGRP: u32 = 0400 >> 3; //0400>>3 read by group
static IWGRP: u32 = 0200 >> 3; //0200>>3 write by group

pub(crate) fn fifo_connection(dltuserinner: &mut DltUserInner) -> Result<(), Error> {
    let mut log_path = LOG_PATH.to_string();
    let mut user_path = USER_PATH.to_string();
    let process_id = process::id().to_string();
    user_path.push_str(&process_id);

    let mode = 0o3777; //read write and execute for owner and for group

    //create tmp/dltpipes path with permissions
    fs::create_dir(DIR);
    let mut perms = fs::metadata(DIR)?.permissions();
    perms.set_mode(mode);
    fs::set_permissions(DIR, perms)?;

    unsafe { libc::unlink(user_path.as_ptr() as *const i8) };

    let filename = CString::new(user_path.clone()).unwrap();

    let fs = unsafe {
        libc::mkfifo(filename.as_ptr(), IREAD | IWRITE | IRGRP | IWGRP) // as_ptr moved here
    };

    //opening path handles
    let mut user_path_handle = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(user_path)?;
    let mut log_path_handle = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_WRONLY | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(log_path)?;

    //add path handles to dltuserinner
    dltuserinner.dlt_user_handle = Some(user_path_handle.into());
    dltuserinner.dlt_log_handle = Some(log_path_handle.into());

    Ok(())
}

mod tests {
    use std::time::Duration;

    use async_std::io::{ReadExt, WriteExt};

    use super::*;

    #[test]
    fn b() {
        let mut dltuserinner = DltUserInner::new(CONFIG).unwrap();
        fifo_connection(&mut dltuserinner);
    }

    #[test]
    fn basicee() {
        let mut dltuserinner = DltUserInner::new(CONFIG).unwrap();
        let res = fifo_connection(&mut dltuserinner).unwrap();
        let mut user_handle = dltuserinner.dlt_user_handle.unwrap();
        user_handle.write(b"hello world");
        thread::sleep(time::Duration::from_secs(3));

        let mut buffer = [0; 1024];

        let res = match user_handle.read(&mut buffer) {
            Ok(bytes_read) => {
                let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("{}", message);
            }
            Err(e) => panic!("Error reading from pipe:{}", e),
        };
    }
}
