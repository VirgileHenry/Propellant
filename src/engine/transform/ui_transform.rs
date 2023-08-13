use std::cell::Cell;

use tree_box::TreeBox;

use crate::{
    CursorPosition,
    engine::renderer::graphic_pipeline::uniform::{object_uniform::ui_model_uniform::UiPosUniformObject, frame_uniform::ui_resolution::UiResolution}
};

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
    position: glam::Vec2,
    relative_position: glam::Vec2,
    size: glam::Vec2,
    relative_size: glam::Vec2,
    anchor: UiAnchor,
    layer: i32,
    resolution: Option<UiResolution>,
    computed_pos: Cell<Option<UiPosUniformObject>>,
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

    pub fn invalidate_pos(&self) {
        self.computed_pos.set(None);
    }

    pub fn recompute_pos(&self, parent: Option<UiPosUniformObject>) -> UiPosUniformObject {
        // entry quad pos will be [0, 0, 1] x [1, 1, 0]
        // need to move that to [pos, pos depth] x [pos + size, pos + size, depth]
        let (ax, ay) = self.anchor.to_values();
        let anchor = glam::Vec2::new(ax, ay);
        let pos = match parent {
            Some(parent_pos) => {
                #[cfg(feature = "debug-features")]
                let resolution = self.resolution.expect("Screen size not set for ui transform. You might missed a UiRequireScreenSizeFlag flag when creating new UI elemnts.");
                #[cfg(not(feature = "debug-features"))]
                let resolution = self.screen_size.unwrap_or(glam::Vec2::new(1080.0, 1920.0));
                let screen_size = glam::Vec2::new(resolution.screen_width, resolution.screen_width);
                let total_size = self.relative_size + self.size / screen_size;
                let total_pos = self.relative_position + self.position / screen_size - anchor * total_size;
                // reset parent depth
                parent_pos.pos.col(2).z = 1.;
                let pos_matrix = parent_pos.pos * glam::Mat3::from_scale_angle_translation(total_size, 0., total_pos);
                pos_matrix.col(2).z = self.depth_to_render_cube(); // override parent depth
                UiPosUniformObject {
                    pos: pos_matrix,
                }
            },
            None => {
                #[cfg(feature = "debug-features")]
                let resolution = self.resolution.expect("Screen size not set for ui transform. You might missed a UiRequireScreenSizeFlag flag when creating new UI elemnts.");
                #[cfg(not(feature = "debug-features"))]
                let resolution = self.screen_size.unwrap_or(glam::Vec2::new(1080.0, 1920.0));
                let screen_size = glam::Vec2::new(resolution.screen_width, resolution.screen_width);
                let total_size = self.relative_size + self.size / screen_size;
                let total_pos = self.relative_position + self.position / screen_size - anchor * total_size;
                let pos_matrix = glam::Mat3::from_scale_angle_translation(total_size, 0., total_pos);
                pos_matrix.col(2).z = self.depth_to_render_cube();
                UiPosUniformObject {
                    pos: pos_matrix,
                }
            },
        };
        self.computed_pos.set(Some(pos));
        pos
    }

    pub fn get_pos(&self) -> Option<UiPosUniformObject> {
        self.computed_pos.get()
    }

    pub fn depth_to_render_cube(&self) -> f32 {
        // [-inf, inf] -> [0, 1]
        const PI_INVERTED: f32 = 1.0 / std::f32::consts::PI;
        0.5 - PI_INVERTED * (self.layer as f32).atan()
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
        layer: i32,
    ) -> UiTransform {
        UiTransform {
            core: TreeBox::from(UiTransformCore {
                position,
                relative_position,
                size,
                relative_size,
                anchor,
                computed_pos: Cell::new(None),
                layer,
                resolution: None,
            }),
        }
    }


    fn invalidate_pos(&mut self) {
        self.core.mutate(|tf| tf.invalidate_pos());
        self.core.mutate_children_rec(|tf| tf.invalidate_pos());
    }

    
    pub fn get_pos(&self) -> UiPosUniformObject {
        // rec get on the parent:
        // parent is none -> identity,
        // else -> parent world pos
        self.core.get_parent_rec(
            |tf| tf.get_pos(),
            |tf, parent_world_pos| tf.recompute_pos(parent_world_pos),
        )
    }

    pub fn child_of(mut self, parent: Option<&UiTransform>) -> UiTransform {
        self.core.set_parent(parent.map(|v| &v.core));
        self.invalidate_pos();
        self
    }

    pub fn ui_contains_cursor(&self, cursor: CursorPosition) -> bool {
        self.core.get(|tf| tf.ui_contains_cursor(cursor))
    }
    // todo : funcs to change the transform

}


