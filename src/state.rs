#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState{
    Meta,
    Cpu,
    Disk,
    Processes,
    Network,
    Sockets,
    Memory,
    ShuttingDown
}