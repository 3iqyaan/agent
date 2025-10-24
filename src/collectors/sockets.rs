use anyhow::Error;
use tokio::sync::mpsc;
use netstat2::*;

use crate::{models::Data, state::AppState, APPSTATE, LSOCKET_DURATION, SOCKET_DURATION};

pub async fn main(tx: mpsc::Sender<Data>) -> Result<(), Error>{
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;

    loop{
        let state: AppState;
        if let Some(appstate) = APPSTATE.get(){
            state = *appstate.read().await;
            if state == AppState::ShuttingDown{ break; } // Graceful Shutdown
        }
        else{
            tokio::time::sleep(SOCKET_DURATION).await;
            continue; // Wait until appstate is available
        }

        let sockets_info = get_sockets_info(af_flags, proto_flags).unwrap(); // Refresh the sockets info
        let mut sock_vec = Vec::new();

        for si in sockets_info {
            match si.protocol_socket_info {
                ProtocolSocketInfo::Tcp(tcp_si) => {
                    sock_vec.push(crate::models::Sockets::Tcp { 
                        local_addr: tcp_si.local_addr.to_string(), 
                        local_port: tcp_si.local_port, 
                        remote_addr: tcp_si.remote_addr.to_string(),
                        remote_port: tcp_si.remote_port,
                        pids: si.associated_pids,
                        state: tcp_si.state.to_string()
                    });
                }
                ProtocolSocketInfo::Udp(udp_si) => {
                    sock_vec.push(crate::models::Sockets::Udp {
                        local_addr: udp_si.local_addr.to_string(),
                        local_port: udp_si.local_port,
                        pid: si.associated_pids
                    });
                }
            }
        }
        tx.send(Data::Sockets(sock_vec)).await?;

        match state{
            AppState::Sockets => {
                tokio::time::sleep(SOCKET_DURATION).await;
            }
            _ => {
                tokio::time::sleep(LSOCKET_DURATION).await; // Refresh slowly
            }
        }
    }
    Ok(())
}