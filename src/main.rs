use std::{sync::{Arc, OnceLock, RwLock}};
use anyhow::Error;
use sysinfo::{
    Disks, Networks, System
};
use tokio::{sync::{mpsc}, time::{Duration}};

use crate::state::AppState;

mod collectors;
mod cli;
mod transmitter;
mod state;
mod models;

pub const MEM_DURATION: Duration = Duration::from_millis(2000);
pub const LMEM_DURATION: Duration = Duration::from_millis(3000);

pub const NET_DURATION: Duration = Duration::from_millis(2120);
pub const LNET_DURATION: Duration = Duration::from_millis(3214);

pub const SOCKET_DURATION: Duration = Duration::from_millis(1120);
pub const LSOCKET_DURATION: Duration = Duration::from_millis(3520);

pub const PROC_DURATION: Duration = Duration::from_millis(3141);
pub const LPROC_DURATION: Duration = Duration::from_millis(5260);

pub const DISK_DURATION: Duration = Duration::from_millis(1512);
pub const LDISK_DURATION: Duration = Duration::from_millis(2893);

pub const MAX_SIZE: usize = 1024;

pub static APPSTATE: OnceLock<Arc<tokio::sync::RwLock<AppState>>> = OnceLock::new();
pub static SYSTEM: OnceLock<Arc<RwLock<System>>> = OnceLock::new();
pub static NETWORKS: OnceLock<Arc<RwLock<Networks>>> = OnceLock::new();
pub static DISKS: OnceLock<Arc<RwLock<Disks>>> = OnceLock::new();

// To decide where we are sending the data to and who we are interacting with.
// If this just a cli, we send data to the cli reciever. Else, we send to the tauri reciever.
pub static IS_CLI: OnceLock<Arc<RwLock<bool>>> = OnceLock::new();  

#[tokio::main]
async fn main() -> Result<(), Error>{
    let power_on: bool = true;

    let is_cli = Arc::new(RwLock::new(true)); // For now this just cli
    IS_CLI.set(is_cli.clone()).unwrap();

    let app_state: Arc<tokio::sync::RwLock<AppState>> = Arc::new(tokio::sync::RwLock::new(AppState::Meta));
    APPSTATE.set(app_state.clone()).unwrap();

    // the channel to the refresher from the helpers
    let (refr_tx, refr_rx) = mpsc::channel(20);

    // the channel to the transmitter from the helpers
    let (tx, rx) = mpsc::channel(200);

    // Initialize the transmitter
    let transmitter_handle = tokio::spawn(async move{
        if let Err(e) = transmitter::main(rx).await{
            eprint!("The transmitter panicked: {:?}", e);
        }
    });

    // Initialize the refresher
    let refr_handle = tokio::spawn(async move{
        if let Err(e) = refresher(refr_rx).await{
            eprint!("The refresher panicked: {:?}", e);
        }
    });
    refr_tx.send(Kind::Init).await?;

    // Initialize the Meta and cpu
    let (tx_clone, rtx_clone) = (tx.clone(), refr_tx.clone());
    let meta_handle = tokio::spawn(async move{
        if let Err(e) = collectors::meta::main(tx_clone, rtx_clone).await{
            eprint!("Meta data collector panicked: {:?}", e);
        }
    });

    // Initialize the Disk
    let (tx_clone, rtx_clone) = (tx.clone(), refr_tx.clone());
    let disk_handle = tokio::spawn(async move{
        if let Err(e) = collectors::disks::main(tx_clone, rtx_clone).await{
            eprint!("Disk data collector panicked: {:?}", e);
        }
    });

    // Initialize Memory
    let (tx_clone, rtx_clone) = (tx.clone(), refr_tx.clone());
    let mem_handle = tokio::spawn(async move{
        if let Err(e) = collectors::memory::main(tx_clone, rtx_clone).await{
            eprint!("Memory data collector panicked: {:?}", e);
        }
    });

    // Initialize the networks
    let (tx_clone, rtx_clone) = (tx.clone(), refr_tx.clone());
    let net_handle = tokio::spawn(async move{
        if let Err(e) = collectors::networks::main(tx_clone, rtx_clone).await{
            eprint!("Network data collector panicked: {:?}", e);
        }
    });

    // Initialize the Sockets
    let tx_clone = tx.clone();
    let socket_handle = tokio::spawn(async move{
        if let Err(e) = collectors::sockets::main(tx_clone).await{
            eprint!("Socket data collector panicked: {:?}", e);
        }
    });

    // Initialize the processes
    let (tx_clone, rtx_clone) = (tx.clone(), refr_tx.clone());
    let proc_handle = tokio::spawn(async move{
        if let Err(e) = collectors::processes::main(tx_clone, rtx_clone).await{
            eprint!("Process data collector panicked: {:?}", e);
        }
    });

    while power_on{
        continue;
    }
    refr_handle.await?;
    meta_handle.await?;
    disk_handle.await?;
    mem_handle.await?;
    net_handle.await?;
    socket_handle.await?;
    proc_handle.await?;
    transmitter_handle.await?;

    Ok(())

}

pub enum Kind{
    Init,
    Cpu,
    Disk,
    Net,
    Proc,
    Meta,
    Memory,
    ShuttingDown
}

pub async fn refresher(mut rx: mpsc::Receiver<Kind>) -> Result<(), Error>{
    let system = Arc::new(RwLock::new(System::new_all()));
    let networks_ = Arc::new(RwLock::new(Networks::new_with_refreshed_list()));
    let disks_ = Arc::new(RwLock::new(Disks::new_with_refreshed_list()));

    loop {
        if let Some(kind) = rx.recv().await{
            match kind{
                Kind::Init => {
                    let _ = SYSTEM.set(system.clone());
                    let _ = NETWORKS.set(networks_.clone());
                    let _ = DISKS.set(disks_.clone());
                }
                Kind::Cpu => {
                    if let Some(sys) = SYSTEM.get(){
                        let mut refd = sys.write().unwrap();
                        refd.refresh_cpu_all();
                    }
                }
                Kind::Meta => {
                    if let Some(sys) = SYSTEM.get(){
                        let mut refd = sys.write().unwrap();
                        refd.refresh_all();
                    }
                }
                Kind::Disk => {
                    if let Some(disks_) = DISKS.get(){
                        let mut refd = disks_.write().unwrap();
                        refd.refresh(true);
                    }
                }
                Kind::Memory => {
                    if let Some(sys) = SYSTEM.get(){
                        let mut refd = sys.write().unwrap();
                        refd.refresh_memory();
                    }
                }
                Kind::Net => {
                    if let Some(net) = NETWORKS.get(){
                        let mut refd = net.write().unwrap();
                        refd.refresh(true);
                    }
                }
                Kind::Proc => {
                    if let Some(sys) = SYSTEM.get(){
                        let mut refd = sys.write().unwrap();
                        refd.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                    }
                }
                Kind::ShuttingDown => {
                    break;
                }
            }
        }
    }
    Ok(())
}