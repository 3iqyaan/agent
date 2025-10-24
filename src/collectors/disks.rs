use tokio::sync::mpsc;
use anyhow::Error;
use crate::{models::{Data, DiskData}, state::AppState, Kind, APPSTATE, DISKS, DISK_DURATION, LDISK_DURATION};

pub async fn main(tx: mpsc::Sender<Data>, refr_tx: mpsc::Sender<Kind>) -> Result<(), Error>{
    loop{
        refr_tx.send(Kind::Disk).await?; // refresh the data

        let state: AppState;
        if let Some(appstate) = APPSTATE.get(){
            state = *appstate.read().await;
            if state == AppState::ShuttingDown{
                break;
            }
        }
        else{
            continue;
        }

        let mut disc_vec = Vec::new();


        if let Some(disks) = DISKS.get(){
            let disks = match disks.read(){
                Ok(data) => data,
                Err(poisoned) => {
                    eprintln!("FATAL: DISKS lock was poisoned!, Recovering");
                    poisoned.into_inner()
                }
            };
            for disk in &*disks{
                disc_vec.push(DiskData { 
                    name: disk.name().to_string_lossy().to_string(), 
                    fs: disk.file_system().to_string_lossy().to_string(), 
                    type_: disk.kind().to_string(), 
                    removable: disk.is_removable(), 
                    loc: disk.mount_point().to_string_lossy().to_string(), 
                    read_only: disk.is_read_only(), 
                    t_space: disk.total_space(), 
                    a_space: disk.available_space(), 
                    t_written: disk.usage().total_written_bytes, 
                    written: disk.usage().written_bytes, 
                    t_read: disk.usage().total_read_bytes, 
                    read: disk.usage().read_bytes 
                })
            }
        }
        tx.send(Data::Disk(disc_vec)).await?;
        
        match state{
            AppState::Disk => {
                tokio::time::sleep(DISK_DURATION).await;
            }
            _ => {
                tokio::time::sleep(LDISK_DURATION).await;
            }
        }
    }
    Ok(())
}