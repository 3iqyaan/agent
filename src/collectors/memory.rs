use anyhow::Error;
use tokio::sync::mpsc;

use crate::{models::{Data, Memory}, state::AppState, Kind, APPSTATE, LMEM_DURATION, MEM_DURATION, SYSTEM};

pub async fn main(tx: mpsc::Sender<Data>, rtx: mpsc::Sender<Kind>) -> Result<(), Error>{
    loop{
        rtx.send(Kind::Proc).await?;

        let mem: Memory;
        let state: AppState;

        if let Some(appstate) = APPSTATE.get(){ // Make sure that Appstate is available
            state = *appstate.read().await;
            if state == AppState::ShuttingDown{ break; } // Graceful Shutdown
        }
        else{
            tokio::time::sleep(MEM_DURATION).await;
            continue; // Wait until appstate is available
        }
        if let Some(sys) = SYSTEM.get(){
            let sys = match sys.read(){
                Ok(data) => data,
                Err(poisoned) => {
                    eprintln!("FATAL: SYSTEM lock was poisoned!, Recovering");
                    poisoned.into_inner()
                }
            };
            mem = Memory{
                t_ram: sys.total_memory(),
                a_ram: sys.available_memory(),
                u_ram: sys.used_memory(),
                t_swap: sys.total_swap(),
                u_swap: sys.used_swap(),
                a_swap: sys.free_swap()
            }
        }
        else{
            continue;
        }
        tx.send(Data::Memory(mem)).await?;
        match state{
            AppState::Memory => {
                tokio::time::sleep(MEM_DURATION).await;
            }
            _ => {
                tokio::time::sleep(LMEM_DURATION).await;
            }
        }
    }
    Ok(())
}