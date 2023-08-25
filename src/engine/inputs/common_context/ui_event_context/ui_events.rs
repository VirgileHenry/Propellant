
use super::CursorPosition;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiEvent {
    MouseMove(CursorPosition),
    MousePrimaryClick,
    MousePrimaryRelease,
}

impl TryFrom<&winit::event::WindowEvent<'_>> for UiEvent {
    type Error = ();
    fn try_from(event: &winit::event::WindowEvent<'_>) -> Result<UiEvent, ()> {
        match event {
            winit::event::WindowEvent::CursorMoved { position, .. } => Ok(UiEvent::MouseMove(CursorPosition::InScreen{
                mouse_x: position.x as f32,
                mouse_y: position.y as f32,
            })),
            winit::event::WindowEvent::CursorLeft { .. } => Ok(UiEvent::MouseMove(CursorPosition::OutOfScreen)),
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                match (state, button) {
                    (winit::event::ElementState::Pressed, winit::event::MouseButton::Left) => Ok(UiEvent::MousePrimaryClick),
                    (winit::event::ElementState::Released, winit::event::MouseButton::Left) => Ok(UiEvent::MousePrimaryRelease),
                    _ => Err(()),
                }
            },
            _ => Err(()),
        }
    }
}