

/// Events to send to the event loop.
#[derive(Debug, Clone)]
pub enum PropellantEvent {
    CloseApplicationRequest,
    SwapchainRecreationRequest,
    SwitchInputContext(u64),
}

