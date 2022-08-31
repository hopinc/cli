#[derive(Debug)]
pub enum HeartbeatManagerEvent {
    Shutdown,
    UpdateInterval(u64),
}
