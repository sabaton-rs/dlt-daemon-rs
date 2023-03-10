use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use async_std::fs::OpenOptions;

use crate::DltUserInner;
use async_std::io::prelude::WriteExt;

pub(crate) async fn mainloop(dlt_user: Arc<Mutex<DltUserInner>>) {
    println!("Mainloop processing started");

    // This is the task for receiving messages from the server
    async_std::task::spawn(async move {
        'incoming_outer: loop {
            //let temp_path = temp
            println!("Incoming task started");

            // TODO: Open the incoming FIFO here and loop over it
            // asynchronously

            //dlt_user.lock().unwrap().dlt_log_handle = Some(outgoing_fifo().unwrap());

            async_std::task::sleep(Duration::from_millis(100)).await;
            // something seriously wrong. Go back to outer loop and
            // try to connect again
            continue 'incoming_outer;
        }
    });

    let rx = dlt_user.lock().unwrap().receiver.clone();
    'outer: loop {
        // attempt to connect to the daemon
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(false)
            .open("/tmp/dlt")
            .await
        {
            println!("Connect successful");
            loop {
                // wait for messages that need to be sent
                if let Ok(message) = rx.recv().await {
                    let bytes = message.as_bytes();
                    if let Err(_e) = file.write_all(bytes.as_slice()).await {
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
        } else {
            // wait for a bit before attempting to connect
            println!("Connect failed. Will retry");
            async_std::task::sleep(Duration::from_millis(200)).await;

            continue;
        }
    }
}
