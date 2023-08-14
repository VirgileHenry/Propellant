use foundry::ComponentTable;
use winit::event_loop::EventLoopProxy;

use crate::{
    PropellantEngine,
    PropellantWindow, PropellantFlag
};

use super::{errors::PResult, consts::PROPELLANT_DEBUG_FEATURES};


/// Events to send to the event loop.
#[derive(Debug, Clone)]
pub enum PropellantEvent {
    CloseApplicationRequest,
    SwapchainRecreationRequest,
    AddEventContext(u64),
    RemoveEventContext(u64),
    HandleEngineFlag(PropellantFlag),
}

impl PropellantEngine {
    pub fn handle_propellant_event(&mut self, event: PropellantEvent, control_flow: &mut winit::event_loop::ControlFlow) {
        match event {
            // engine requested stop
            PropellantEvent::CloseApplicationRequest => control_flow.set_exit(),
            PropellantEvent::SwapchainRecreationRequest => {
                // get to the window, and ask swap chain recreation.
                match self.world.get_singleton_mut::<PropellantWindow>() {
                    Some(window) => {
                        match window.recreate_swapchain() {
                            Ok(_) => {},
                            Err(e) => println!("{e}"),
                        };
                    },
                    None => {},
                }
            },
            PropellantEvent::AddEventContext(ctx_id) => self.add_input_context(ctx_id),
            PropellantEvent::RemoveEventContext(ctx_id) => self.remove_input_context(ctx_id),
            PropellantEvent::HandleEngineFlag(flag) => match self.handle_flag(flag) {
                Ok(_) => {},
                Err(e) => if PROPELLANT_DEBUG_FEATURES {
                    println!("[PROPELLANT DEBUG] Error while handling engine flag: {e}");
                },
            },
        }
    }
}


pub struct PropellantEventSender {
    proxy: EventLoopProxy<PropellantEvent>,
}

impl PropellantEventSender {
    pub fn new(proxy: EventLoopProxy<PropellantEvent>) -> Self {
        Self {
            proxy,
        }
    }

    pub fn send(&self, event: PropellantEvent) -> PResult<()> {
        self.proxy.send_event(event)?;
        Ok(())
    }
}

pub trait PropellantEventSenderExt {
    fn send_event(&self, event: PropellantEvent) -> PResult<()>;
    fn send_flag(&self, flag: PropellantFlag) -> PResult<()>;
}

impl PropellantEventSenderExt for ComponentTable {
    fn send_event(&self, event: PropellantEvent) -> PResult<()> {
        match self.get_singleton::<PropellantEventSender>() {
            Some(sender) => sender.send(event),
            None => {
                if PROPELLANT_DEBUG_FEATURES {
                    println!("[PROPELLANT DEBUG] Event sender: no event sender found. Event not sent.");
                }
                Ok(())
            }
        }
    }

    fn send_flag(&self, flag: PropellantFlag) -> PResult<()> {
        match self.get_singleton::<PropellantEventSender>() {
            Some(sender) => sender.send(PropellantEvent::HandleEngineFlag(flag)),
            None => {
                if PROPELLANT_DEBUG_FEATURES {
                    println!("[PROPELLANT DEBUG] Event sender: no event sender found. Event not sent.");
                }
                Ok(())
            }
        }
    }
}