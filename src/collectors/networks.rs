use anyhow::Error;
use tokio::sync::mpsc;

use crate::{models::{Data, Networks}, state::AppState, Kind, APPSTATE, LNET_DURATION, NETWORKS, NET_DURATION};

pub async fn main(tx: mpsc::Sender<Data>, refr_tx: mpsc::Sender<Kind>) -> Result<(), Error>{
    loop{
        refr_tx.send(Kind::Net).await?;
        
        let mut net_vec = Vec::new();
        let state: AppState;

        if let Some(appstate) = APPSTATE.get(){ // Make sure that Appstate is available
            state = *appstate.read().await;
            if state == AppState::ShuttingDown{ break; } // Graceful Shutdown
        }
        else{
            tokio::time::sleep(NET_DURATION).await;
            continue; // Wait until appstate is available
        }

        if let Some(networks) = NETWORKS.get(){
            let networks = match networks.read(){
                Ok(data) => data,
                Err(poisoned) => {
                    eprintln!("FATAL: NETWORKS lock was poisoned!, Recovering");
                    poisoned.into_inner()
                }
            };
            for network in &*networks{
                net_vec.push(Networks{
                    name: network.0.to_string(),
                    t_down: network.1.total_received(),
                    down: network.1.received(),
                    t_up: network.1.total_transmitted(),
                    up: network.1.transmitted(),
                    t_packet_rx: network.1.total_packets_received(),
                    packet_rx: network.1.packets_transmitted(),
                    t_packet_tx: network.1.total_packets_transmitted(),
                    packet_tx: network.1.packets_received(),
                    t_err_rx: network.1.total_errors_on_received(),
                    err_rx: network.1.errors_on_received(),
                    t_err_tx: network.1.total_errors_on_transmitted(),
                    err_tx: network.1.errors_on_transmitted()
                });
            }
        }
        
        tx.send(Data::Networks(net_vec)).await?;
        
        match state{
            AppState::Network => {
                tokio::time::sleep(NET_DURATION).await;
            }
            _ => {
                tokio::time::sleep(LNET_DURATION).await; // Slower refresh rate
            }
        }
    }
    Ok(())
}