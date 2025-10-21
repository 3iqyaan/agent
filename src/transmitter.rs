use std::sync::Arc;

use bincode;
use anyhow::Error;
use sysinfo::MINIMUM_CPU_UPDATE_INTERVAL;
use tokio::sync::{mpsc, Semaphore};

use crate::models::Data;

pub async fn main(mut rx: mpsc::Receiver<Data>) -> Result<(), Error>{
    let semaphore = Arc::new(Semaphore::new(6));
    let mut batch: Vec<Data> = Vec::with_capacity(12);
    while let Ok(data) = rx.try_recv(){
        batch.push(data);
    }

    if !batch.is_empty(){
        let permit = match semaphore.clone().acquire_owned().await{
            Ok(perm) => perm,
            Err(e) => {
                eprintln!("An owned Permit was not dropped or the holding it panicked");
                return Err(anyhow::anyhow!(e))
            }
        };
        let msg = std::mem::take(&mut batch);
        tokio::spawn(async move{
            transmit(&msg).await;
            drop(permit);
        });
    }
    else{
        tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL/2).await;
    }
    Ok(())
}

async fn transmit(payload: &Vec<Data>){
    let packet = bincode::serialize(payload).expect("Error: Serialization failed");


}