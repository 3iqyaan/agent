use iceoryx2::{node as ice_node, service::ipc};
use iceoryx2::port::publisher;
use anyhow::{Error, Ok};
use sysinfo::MINIMUM_CPU_UPDATE_INTERVAL;
use tokio::sync::{mpsc};

use crate::{models::{Data, TelemetryKind, Telemetry}, MAX_SIZE};

pub async fn main(mut rx: mpsc::Receiver<Data>) -> Result<(), Error>{

    let mut binary: Vec<u8> = Vec::with_capacity(MAX_SIZE * 2);

    let node = ice_node::NodeBuilder::new()
        .name(&"AwareAgent".try_into()?)
        .create::<ipc::Service>()?;

    let service = node.service_builder(&"Telemetry".try_into()?)
        .publish_subscribe::<Telemetry>()
        .open_or_create()?;

    let writer  = service.publisher_builder().create()?;

    const CYCLES: u32 = 24;

    let mut status = true;

    while status{
        let mut processed = 0;
        while processed < CYCLES{
            match rx.try_recv(){
                std::result::Result::Ok(data) => {
                    if !handle_data(data, &writer, &mut binary)?{
                        status = false;
                        break;
                    }
                    processed += 1;
                }
                Err(_) => break
            }
        }
        node.wait(MINIMUM_CPU_UPDATE_INTERVAL);


    }
    Ok(())
}

fn handle_data(data: Data, writer: &publisher::Publisher<ipc::Service, Telemetry, ()>, binary: &mut Vec<u8>) -> Result<bool, Error>{
    let mut ret = true;
    let mut buf = [0u8; MAX_SIZE];

    match data{
        Data::ShuttingDown =>  {
            ret = false;
        }
        Data::Cpus(vec_cpus) => {
            bincode::serialize_into(binary as &mut Vec<u8>, &vec_cpus)?;
            for chunk in binary.chunks(MAX_SIZE) {
                buf[..chunk.len()].copy_from_slice(chunk);
                let mut sample = writer.loan()?;
                sample.data = buf;
                sample.kind = TelemetryKind::Cpus;
                sample.send()?;
            }
            
        }
        Data::Disk(vec_disk) => {
            bincode::serialize_into(binary as &mut Vec<u8>, &vec_disk)?;
            for chunk in binary.chunks(MAX_SIZE) {
                buf[..chunk.len()].copy_from_slice(chunk);
                let mut sample = writer.loan()?;
                sample.data = buf;
                sample.kind = TelemetryKind::Disk;
                sample.send()?;
            }
        }
        Data::Memory(vec_mem) => {
            bincode::serialize_into(binary as &mut Vec<u8>, &vec_mem)?;                    for chunk in binary.chunks(MAX_SIZE) {
                buf[..chunk.len()].copy_from_slice(chunk);
                let mut sample = writer.loan()?;
                sample.data = buf;
                sample.kind = TelemetryKind::Memory;
                sample.send()?;
            }
        }
        Data::Process(vec_proc) => {
            bincode::serialize_into(binary as &mut Vec<u8>, &vec_proc)?;
            for chunk in binary.chunks(MAX_SIZE) {
                buf[..chunk.len()].copy_from_slice(chunk);
                let mut sample = writer.loan()?;
                sample.data = buf;
                sample.kind = TelemetryKind::Process;
                sample.send()?;
            }
        }
        Data::Sockets(vec_sock) => {
            bincode::serialize_into(binary as &mut Vec<u8>, &vec_sock)?;
            for chunk in binary.chunks(MAX_SIZE) {
                buf[..chunk.len()].copy_from_slice(chunk);
                let mut sample = writer.loan()?;
                sample.data = buf;
                sample.kind = TelemetryKind::Sockets;
                sample.send()?;
            }
        }
        Data::Meta(meta) => {
            bincode::serialize_into(binary as &mut Vec<u8>, &meta)?;
            for chunk in binary.chunks(MAX_SIZE) {
                buf.fill(0);
                buf[..chunk.len()].copy_from_slice(chunk);
                let mut sample = writer.loan()?;
                sample.data = buf;
                sample.kind = TelemetryKind::Meta;
                sample.send()?;
            }
        }
        Data::Networks(vec_net) => {
            bincode::serialize_into(binary as &mut Vec<u8>, &vec_net)?;                    for chunk in binary.chunks(MAX_SIZE) {
                buf[..chunk.len()].copy_from_slice(chunk);
                let mut sample = writer.loan()?;
                sample.data = buf;
                sample.kind = TelemetryKind::Networks;
                sample.send()?;
            }
        }
    }
    Ok(ret)
}