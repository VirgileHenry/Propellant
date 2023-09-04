use crate::engine::ui::ui_transform::UiTransform;

use super::ObjectUniform;

#[repr(C)] // important for any data we send to the gpu
#[allow(unused)] // we don't use the fields directly, but they are used by the gpu
#[derive(Debug, Clone, Copy)]
pub struct UiPosUniformObject {
    pub col1: glam::Vec4,
    pub col2: glam::Vec4,
    pub col3: glam::Vec4,
}

impl UiPosUniformObject {
    pub fn set_depth(&mut self, depth: f32) {
        self.col3.z = depth;
    }

    pub fn as_matrix(&self) -> glam::Mat3 {
        glam::Mat3::from_cols(
            self.col1.truncate(),
            self.col2.truncate(),
            self.col3.truncate()
        )
    }

    pub fn screen_size(&self, screen_size: glam::Vec2) -> glam::Vec2 {
        glam::vec2(self.col1.x * screen_size.x, self.col2.y * screen_size.y)
    }

    pub fn size(&self) -> glam::Vec2 {
        glam::vec2(self.col1.x, self.col2.y)
    }

    pub fn offset(&self) -> glam::Vec2 {
        // pos mat * [-1, -1, 1] gives the top left corner
        glam::vec2(-self.col1.x + self.col3.x, -self.col2.y + self.col3.y)
    }
}

impl From<(glam::Mat3, f32)> for UiPosUniformObject {
    fn from((mat, depth): (glam::Mat3, f32)) -> Self {
        let col3 = glam::Vec4::new(mat.z_axis.x, mat.z_axis.y, depth, 0.0);
        UiPosUniformObject {
            col1: (mat.x_axis, 0.0).into(),
            col2: (mat.y_axis, 0.0).into(),
            col3,
        }
    }
}

impl Default for UiPosUniformObject {
    fn default() -> Self {
        UiPosUniformObject {
            col1: glam::Vec4::new(1.0, 0.0, 0.0, 0.0),
            col2: glam::Vec4::new(0.0, 1.0, 0.0, 0.0),
            col3: glam::Vec4::new(0.0, 0.0, 1.0, 0.0),
        }
    }
}

impl ObjectUniform for UiPosUniformObject {
    type FromComponent = UiTransform;
    fn set_uniform(transform: &Self::FromComponent, write_to_buf: &mut dyn FnMut(&[Self], usize), instance_count: usize) {
        for i in 0..instance_count {
            write_to_buf(&[transform.get_pos()], i);
        }
    }
}