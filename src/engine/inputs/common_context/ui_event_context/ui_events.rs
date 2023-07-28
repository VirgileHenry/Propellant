use crate::engine::renderer::graphics_pipeline::uniform::frame_uniform::ui_resolution::UiResolution;

use super::CursorPosition;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiEvent {
    MouseMove(CursorPosition),
}

impl TryFrom<(&winit::event::WindowEvent<'_>, &UiResolution)> for UiEvent {
    type Error = ();
    fn try_from((event, res): (&winit::event::WindowEvent<'_>, &UiResolution)) -> Result<UiEvent, ()> {
        match event {
            winit::event::WindowEvent::CursorMoved { position, .. } => Ok(UiEvent::MouseMove(CursorPosition::InScreen{
                mouse_x: position.x as f32,
                mouse_y: position.y as f32,
                screen_width: res.screen_width,
                screen_height: res.screen_height,
                ui_res: res.resolution,
            })),
            winit::event::WindowEvent::CursorLeft { .. } => Ok(UiEvent::MouseMove(CursorPosition::OutOfScreen)),
            _ => Err(()),
        }
    }
}