pub(crate) mod input_handler;
pub(crate) mod input_listener;


/// Events to send to the event loop.
#[derive(Debug, Clone)]
pub enum PropellantEvent {
    CloseApplicationRequest,
    SwapchainRecreationRequest,
}

