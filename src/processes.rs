use anyhow::Error;
use tokio::sync::mpsc;

use crate::{models::{Data, Process}, state::AppState, Kind, APPSTATE, LPROC_DURATION, PROC_DURATION, SYSTEM};

pub async fn main(tx: mpsc::Sender<Data>, rtx: mpsc::Sender<Kind>) -> Result<(), Error>{
    loop{
        rtx.send(Kind::Proc).await?;

        let mut proc_vec = Vec::new();
        let state: AppState;

        if let Some(appstate) = APPSTATE.get(){ // Make sure that Appstate is available
            state = *appstate.read().await;
            if state == AppState::ShuttingDown{ break; } // Graceful Shutdown
        }
        else{
            tokio::time::sleep(PROC_DURATION).await;
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
            for proc in sys.processes(){
                proc_vec.push(Process{
                    pid: proc.0.as_u32(),
                    name: proc.1.name().to_string_lossy().to_string(),
                    exe: match proc.1.exe(){
                        Some(path) => path.to_string_lossy().to_string(),
                        None => "".to_string()
                    },
                    cpu: proc.1.cpu_usage(),
                    mem: proc.1.memory(),
                    status: proc.1.status().to_string(),
                    cmd: proc.1.cmd().into_iter().map(|cmd| cmd.to_string_lossy().to_string()).collect(),
                    parent: match proc.1.parent(){
                        Some(pid) => Some(pid.as_u32()),
                        None => None
                    },
                    user_id: match proc.1.user_id(){
                        Some(uid) => Some(uid.to_string()),
                        None => None
                    }
                });
            }
        }
        tx.send(Data::Process(proc_vec)).await?;
        match state{
            AppState::Processes => {
                tokio::time::sleep(PROC_DURATION).await;
            }
            _ => {
                tokio::time::sleep(LPROC_DURATION).await;
            }
        }
    }
    Ok(())
}