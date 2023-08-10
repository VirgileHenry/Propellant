use tree_box::TreeBox;

use crate::{CursorPosition, engine::renderer::graphic_pipeline::uniform::object_uniform::ui_model_uniform::UiPosUniformObject};

#[derive(Debug, Clone, Copy)]
pub enum UiAnchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl UiAnchor {
    pub fn to_values(self) -> (f32, f32) {
        // first value is 0, 0.5, 1 for left, center, right
        // seocond value is 0, 0.5, 1 for top, center, bottom
        match self {
            UiAnchor::TopLeft => (0., 0.),
            UiAnchor::Top => (0.5, 0.),
            UiAnchor::TopRight => (1., 0.),
            UiAnchor::Left => (0., 0.5),
            UiAnchor::Center => (0.5, 0.5),
            UiAnchor::Right => (1., 0.5),
            UiAnchor::BottomLeft => (0., 1.),
            UiAnchor::Bottom => (0.5, 1.),
            UiAnchor::BottomRight => (1., 1.),
        }
    }
}

pub struct UiTransformCore {
    pub position: glam::Vec2,
    pub relative_position: glam::Vec2,
    pub size: glam::Vec2,
    pub relative_size: glam::Vec2,
    pub anchor: UiAnchor,
}

impl UiTransformCore {
    pub fn ui_contains_cursor(&self, cursor: CursorPosition) -> bool {
        match cursor {
            CursorPosition::OutOfScreen => false,
            CursorPosition::InScreen { mouse_x, mouse_y, screen_width, screen_height, ui_res } => {
                // compute widget screen space
                let tx = self.position.x * ui_res + self.relative_position.x * screen_width;
                let ty = self.position.y * ui_res + self.relative_position.y * screen_height;
                let tw = self.size.x * ui_res + self.relative_size.x * screen_width;
                let th = self.size.y * ui_res + self.relative_size.y * screen_height;
                let (ax, ay) = self.anchor.to_values();
                let tx = tx - ax * tw;
                let ty = ty - ay * th;
                // check if cursor is in widget screen space
                mouse_x >= tx && mouse_x <= tx + tw && mouse_y >= ty && mouse_y <= ty + th
            }
        }
    }

    pub fn to_uniform(&self) -> UiPosUniformObject {
        let (ax, ay) = self.anchor.to_values();
        let anchor = glam::Vec2::new(ax, ay);
        UiPosUniformObject {
            position: self.position,
            relative_position: self.relative_position,
            size: self.size,
            relative_size: self.relative_size,
            anchor,
        }
    }
}

pub struct UiTransform {
    core: TreeBox<UiTransformCore>,
}

impl UiTransform {
    /// Creates a new transform for ui.
    /// This is still quite dangerous : any world invalidation may lead to matrix recompute and thus
    /// to a loss of the ui information.
    pub fn new(
        position: glam::Vec2,
        relative_position: glam::Vec2,
        size: glam::Vec2,
        relative_size: glam::Vec2,
        anchor: UiAnchor,
    ) -> UiTransform {
        UiTransform {
            core: TreeBox::from(UiTransformCore {
                position,
                relative_position,
                size,
                relative_size,
                anchor,
            }),
        }
    }

    pub fn child_of(mut self, parent: Option<&UiTransform>) -> UiTransform {
        self.core.set_parent(parent.map(|v| &v.core));
        self
    }

    pub fn ui_contains_cursor(&self, cursor: CursorPosition) -> bool {
        self.core.get(|tf| tf.ui_contains_cursor(cursor))
    }

    pub fn to_uniform(&self) -> UiPosUniformObject {
        self.core.get(|tf| tf.to_uniform())
    }

}


