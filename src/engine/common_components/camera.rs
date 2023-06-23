
/// Camera component. Tells through a projection matrix how to project the world on the screen.
/// This component does not carry a transform, as it is not a game object.
/// To be used by the renderer, it must be attached to a game object that also have a transform.
pub struct Camera {
    projection_matrix: glam::Mat4,
    is_main: bool,
}

impl Camera {
    /// Create a new main camera, with a perspective projection matrix.
    pub fn main_perspective(screen_height: f32, screen_width: f32, z_near: f32, z_far: f32, fov_y_radians: f32) -> Camera {
        let proj = glam::Mat4::perspective_rh(fov_y_radians, screen_height / screen_width, z_near, z_far);
        Camera {
            projection_matrix: proj * glam::Mat4::from_scale(glam::vec3(1., -1., 1.)), // flip the y axis
            is_main: true,
        }
    }

    /// Create a new secondary camera, with a perspective projection matrix.
    pub fn secondary_perspective(screen_height: f32, screen_width: f32, z_near: f32, z_far: f32, fov_y_radians: f32) -> Camera {
        let proj = glam::Mat4::perspective_rh_gl(fov_y_radians, screen_height / screen_width, z_near, z_far);
        Camera {
            projection_matrix: proj,
            is_main: false,
        }
    }

    /// Create a new main camera, with an orthographic projection matrix.
    pub fn main_orthographic(screen_height: f32, screen_width: f32, z_near: f32, z_far: f32) -> Camera {
        let proj = glam::Mat4::orthographic_rh_gl(0., screen_width, 0., screen_height, z_near, z_far);
        Camera {
            projection_matrix: proj,
            is_main: true,
        }
    }

    /// Create a new secondary camera, with an orthographic projection matrix.
    pub fn secondary_orthographic(screen_height: f32, screen_width: f32, z_near: f32, z_far: f32) -> Camera {
        let proj = glam::Mat4::orthographic_rh_gl(0., screen_width, 0., screen_height, z_near, z_far);
        Camera {
            projection_matrix: proj,
            is_main: false,
        }
    }

    /// Returns wheter this camera is the main camera or not.
    pub fn is_main(&self) -> bool {
        self.is_main
    }

    /// Get the projection matrix of this camera.
    pub fn projection_matrix(&self) -> glam::Mat4 {
        self.projection_matrix
    }
}