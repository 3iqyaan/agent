use tokio::sync::mpsc;
use sysinfo::{System, MINIMUM_CPU_UPDATE_INTERVAL};
use anyhow::Error;

use crate::{models::{Cpus, Data, Meta}, state::AppState, Kind, APPSTATE, SYSTEM};

pub async fn main(tx: mpsc::Sender<Data>, refr_tx: mpsc::Sender<Kind>) -> Result<(), Error>{
    loop{
        refr_tx.send(Kind::Cpu).await?; // refresh the data

        let state: AppState;
        if let Some(appstate) = APPSTATE.get(){
            state = *appstate.read().await;
            if state == AppState::ShuttingDown{ break;}
        }
        else{
            tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL).await;
            continue; // wait until appstate is available
        }
        let mut vec_cpu = Vec::new();

        let (meta_data, cpu_data) = {
            if let Some(sys) = SYSTEM.get(){
                let ref sys = match sys.read(){
                    Ok(data) => data,
                    Err(poisoned) => {
                        eprintln!("FATAL: SYSTEM lock was poisoned!, Recovering");
                        poisoned.into_inner()
                    }
                };

                let m = Meta{
                    t_mem: sys.total_memory(),
                    t_swap: sys.total_swap(),
                    name: System::name().unwrap_or("".to_string()),
                    kernel: System::kernel_version().unwrap_or("".to_string()),
                    os: System::os_version().unwrap_or("".to_string()),
                    host_name: System::host_name().unwrap_or("".to_string()),
                    n_cpu: sys.cpus().len(),
                    n_proc: sys.processes().len(),
                    glob_cpu: sys.global_cpu_usage()
                };
                for cpu in sys.cpus(){
                    vec_cpu.push(Cpus{
                        brand: cpu.brand().to_string(),
                        cpu_name: cpu.name().to_string(),
                        cpu_per: cpu.cpu_usage(),
                        freq: cpu.frequency()
                    });
                }
                (m, vec_cpu)
            }
            else{
                continue;
            }
        };
        match state {
            AppState::Meta => {
                tx.send(Data::Meta(meta_data)).await?;
                tx.send(Data::Cpus(cpu_data)).await?;
                tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL).await; 
            }
            AppState::Cpu => {
                tx.send(Data::Cpus(cpu_data)).await?;
                tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL).await;
            }
            _ => { // We still need to send the data, but at slower refresh rates.
                tx.send(Data::Cpus(cpu_data)).await?;
                tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL*2).await;
            }
        }
    }
    Ok(())
}

