

pub struct Camera {
    projection_matrix: glam::Mat4,
    is_main: bool,
}

impl Camera {
    pub fn main(screen_height: f32, screen_width: f32, z_near: f32, z_far: f32, fov_y_radians: f32) -> Camera {
        let proj = glam::Mat4::perspective_rh_gl(fov_y_radians, screen_height / screen_width, z_near, z_far);
        Camera {
            projection_matrix: proj,
            is_main: true,
        }
    }

    pub fn secondary(screen_height: f32, screen_width: f32, z_near: f32, z_far: f32, fov_y_radians: f32) -> Camera {
        let proj = glam::Mat4::perspective_rh_gl(fov_y_radians, screen_height / screen_width, z_near, z_far);
        Camera {
            projection_matrix: proj,
            is_main: false,
        }
    }

    pub fn is_main(&self) -> bool {
        self.is_main
    }

    pub fn projection_matrix(&self) -> glam::Mat4 {
        self.projection_matrix
    }
}