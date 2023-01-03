use std::{time::Duration, sync::{Arc, Mutex}};

use async_std::fs::OpenOptions;

use crate::{DltUser, DltUserInner};
use async_std::io::prelude::WriteExt;


pub (crate) async fn mainloop(dlt_user: Arc<Mutex<DltUserInner>>) {
    println!("Mainloop processing started");
    let rx = dlt_user.lock().unwrap().receiver.clone();
    'outer: loop {
        // attempt to connect to the daemon
        if let Ok(mut file) = OpenOptions::new().write(true).create(false).open("/tmp/dlt").await {
            println!("Connect successful");
            loop {
                // wait for messages that need to be sent
                if let Ok(message) = rx.recv().await {
                    let bytes = message.as_bytes();
                    if let Err(e) = file.write_all(bytes.as_slice()).await {
                        println!("Error writing message to file");
                        async_std::task::sleep(Duration::from_millis(100)).await;
                        // something seriously wrong. Go back to outer loop and
                        // try to connect again
                        continue 'outer;
                    }

                } else {
                    panic!("Receiving message from channel")
                }
            }
        }
         else {
            // wait for a bit before attempting to connect
            println!("Connect failed. Will retry");
            async_std::task::sleep(Duration::from_millis(200)).await;

            continue;
        }
    }
}