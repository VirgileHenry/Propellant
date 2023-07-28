use crate::{Transform, CursorPosition};

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


/// The ui transform for ui elements.
/// The matrix is used to store all the information about the ui element, like so:
/// x, rx, 0, ax, -> pos x and relative x, anchor x
/// y, ry, 0, ay, -> pos y and relative y, anchor y
/// w, rw, 0, 0, -> width and relative width
/// h, rh, 0, 0, -> height and relative height
pub type UiTransform = Transform;

impl UiTransform {
    /// Creates a new transform for ui.
    /// This is still quite dangerous : any world invalidation may lead to matrix recompute and thus
    /// to a loss of the ui information.
    pub fn ui_new(
        position: glam::Vec2,
        relative_position: glam::Vec2,
        size: glam::Vec2,
        relative_size: glam::Vec2,
        anchor: UiAnchor,
    ) -> UiTransform {
        let mut result = UiTransform::origin();
        let (anchor_x, anchor_y) = anchor.to_values();
        let ui_matrix = glam::Mat4::from_cols_array_2d(&[
            [position.x, position.y, size.x, size.y],
            [relative_position.x, relative_position.y, relative_size.x, relative_size.y],
            [0.0, 0.0, 0.0, 0.0],
            [anchor_x, anchor_y, 0.0, 0.0],
        ]);
        unsafe { result.set_world_matrix(ui_matrix) };
        result
    }

    pub fn ui_contains_cursor(&self, cursor: CursorPosition) -> bool {
        match cursor {
            CursorPosition::OutOfScreen => false,
            CursorPosition::InScreen { mouse_x, mouse_y, screen_width, screen_height, ui_res } => {
                let (x, y) = (self.world_pos().col(0).x, self.world_pos().col(0).y);
                let (w, h) =  (self.world_pos().col(0).z, self.world_pos().col(0).w);
                let (rx, ry) = (self.world_pos().col(1).x, self.world_pos().col(1).y);
                let (rw, rh) = (self.world_pos().col(1).z, self.world_pos().col(1).w);
                let (ax, ay) = (self.world_pos().col(3).x, self.world_pos().col(3).y);
                let r = ui_res;
                // compute widget screen space
                let tx = x * r + rx * screen_width;
                let ty = y * r + ry * screen_height;
                let tw = w * r + rw * screen_width;
                let th = h * r + rh * screen_height;
                let tx = tx - ax * tw;
                let ty = ty - ay * th;
                // check if cursor is in widget screen space
                mouse_x >= tx && mouse_x <= tx + tw && mouse_y >= ty && mouse_y <= ty + th
            }
        }
    }

}


