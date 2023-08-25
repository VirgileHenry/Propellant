use std::cell::Cell;

use tree_box::TreeBox;

use crate::engine::{
    renderer::graphic_pipeline::uniform::object_uniform::ui_model_uniform::UiPosUniformObject,
    ui::ui_resolution::UiResolution
};
#[cfg(feature = "inputs")]
use crate::CursorPosition;

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
            UiAnchor::TopLeft => (1., 1.),
            UiAnchor::Top => (0., 1.),
            UiAnchor::TopRight => (-1., 1.),
            UiAnchor::Left => (1., 0.),
            UiAnchor::Center => (0., 0.),
            UiAnchor::Right => (-1., 0.),
            UiAnchor::BottomLeft => (1., -1.),
            UiAnchor::Bottom => (0., -1.),
            UiAnchor::BottomRight => (-1., -1.),
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
    pub fn invalidate_pos(&self) {
        self.computed_pos.set(None);
    }

    pub fn recompute_pos(&self, parent: Option<UiPosUniformObject>) -> UiPosUniformObject {
        // entry quad pos will be [-1, -1, 1] x [1, 1, 1]
        // need to move that to [pos, pos depth] x [pos + size, pos + size, depth]
        let (ax, ay) = self.anchor.to_values();
        let anchor = glam::Vec2::new(ax, ay);
        let pos = match parent {
            Some(parent_pos) => {
                #[cfg(feature = "debug-features")]
                let resolution = self.resolution.expect("Screen size not set for ui transform. You might missed a UiRequireScreenSizeFlag flag when creating new UI elemnts.");
                #[cfg(not(feature = "debug-features"))]
                let resolution = self.resolution.unwrap_or(UiResolution::default());
                let scale_factor = resolution.scale_factor() / parent_pos.size();
                let total_size = self.relative_size + self.size * scale_factor;
                let total_pos = -glam::Vec2::ONE + (self.relative_position + self.position * scale_factor) * 2. + anchor * total_size;
                let mat = glam::Mat3::from_scale_angle_translation(total_size, 0., total_pos);
                (parent_pos.as_matrix() * mat, self.depth_to_render_cube()).into()
            },
            None => {
                #[cfg(feature = "debug-features")]
                let resolution = self.resolution.expect("Screen size not set for ui transform. You might missed a UiRequireScreenSizeFlag flag when creating new UI elemnts.");
                #[cfg(not(feature = "debug-features"))]
                let resolution = self.resolution.unwrap_or(UiResolution::default());
                let scale_factor = resolution.scale_factor();
                let total_size = self.relative_size + self.size * scale_factor;
                let total_pos = -glam::Vec2::ONE + (self.relative_position + self.position * scale_factor) * 2. + anchor * total_size;
                let mat = glam::Mat3::from_scale_angle_translation(total_size, 0., total_pos);
                (mat, self.depth_to_render_cube()).into()
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

    pub fn set_ui_resolution(&mut self, resolution: UiResolution) {
        self.resolution = Some(resolution);
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

    pub fn set_ui_resolution(&mut self, resolution: UiResolution) {
        self.core.mutate(|tf| tf.set_ui_resolution(resolution));
        self.invalidate_pos();
    }

    pub fn child_of(mut self, parent: Option<&UiTransform>) -> UiTransform {
        self.core.set_parent(parent.map(|v| &v.core));
        self.invalidate_pos();
        self
    }

    // todo : funcs to change the transform

}


#[cfg(feature = "inputs")]
impl UiTransform {
    pub fn ui_contains_cursor(&self, _cursor: CursorPosition) -> bool {
        self.core.get(|_tf| {
            // todo


            false
        })
    }
}