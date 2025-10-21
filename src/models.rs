use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Data{
    Meta(Meta),
    Disk(Vec<DiskData>),
    Networks(Vec<Networks>),
    Sockets(Vec<Sockets>),
    Cpus(Vec<Cpus>),
    Process(Vec<Process>),
    Memory(Memory)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Memory{
    pub t_ram: u64,
    pub u_ram: u64,
    pub a_ram: u64,
    pub t_swap: u64,
    pub u_swap: u64,
    pub a_swap: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta{
    pub t_mem: u64,
    pub t_swap: u64,
    pub name: String,
    pub kernel: String,
    pub os: String,
    pub host_name: String,
    pub n_cpu: usize,
    pub n_proc: usize,
    pub glob_cpu: f32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiskData{
    pub name: String,
    pub fs: String,
    pub type_: String,
    pub removable: bool,
    pub loc: String,
    pub read_only: bool,
    pub t_space: u64,
    pub a_space: u64,
    pub t_written: u64,
    pub written: u64,
    pub t_read: u64,
    pub read: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Networks{
    pub name: String,
    pub t_down: u64,
    pub down: u64,
    pub t_up: u64,
    pub up: u64,
    pub t_packet_rx: u64,
    pub packet_rx: u64,
    pub t_packet_tx: u64,
    pub packet_tx: u64,
    pub t_err_rx: u64,
    pub err_rx: u64,
    pub t_err_tx: u64,
    pub err_tx: u64
}

#[derive(Serialize, Deserialize, Debug)]
#[repr(u8)]
pub enum Sockets{
    Tcp{
        local_addr: String,
        local_port: u16,
        remote_addr: String,
        remote_port: u16,
        pids: Vec<u32>,
        state: String
    },
    Udp{
        local_addr: String,
        local_port: u16,
        pid: Vec<u32>
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cpus{
    pub brand: String,
    pub cpu_name: String,
    pub cpu_per: f32,
    pub freq: u64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Process{
    pub pid: u32,
    pub name: String,
    pub exe: String,
    pub cpu: f32,
    pub mem: u64,
    pub status: String,
    pub cmd: String,
    pub parent: Option<u32>,
    pub user_id: Option<String>
}