

const VK_TO_PROP_SPACE: glam::Mat4 = glam::Mat4{
    x_axis: glam::Vec4::new(1., 0., 0., 0.),
    y_axis: glam::Vec4::new(0., -1., 0., 0.),
    z_axis: glam::Vec4::new(0., 0., 1., 0.),
    w_axis: glam::Vec4::new(0., 0., 0., 1.),
};

enum CameraTypeProperty {
    /// Properties for a perspective camera
    Perspective{
        aspect_ratio: f32,
        fov_y_radians: f32,
        z_near: f32,
        z_far: f32,
    },
    /// height, width, z_near, z_far
    Orthographic{
        aspect_ratio: f32,
        view_height: f32,
        z_near: f32,
        z_far: f32,
    },
}

/// Camera component. Tells through a projection matrix how to project the world on the screen.
/// This component does not carry a transform, as it is not a game object.
/// To be used by the renderer, it must be attached to a game object that also have a transform.
pub struct Camera {
    properties: CameraTypeProperty,
    projection_matrix: glam::Mat4,
    is_main: bool,
}

impl Camera {
    /// Create a new main camera, with a perspective projection matrix.
    pub fn main_perspective(screen_height: f32, screen_width: f32, z_near: f32, z_far: f32, fov_y_radians: f32) -> Camera {
        let aspect_ratio = screen_width / screen_height;
        let proj = glam::Mat4::perspective_rh(fov_y_radians, aspect_ratio, z_near, z_far);
        Camera {
            properties: CameraTypeProperty::Perspective{
                aspect_ratio, fov_y_radians, z_near, z_far,
            },
            projection_matrix: proj * VK_TO_PROP_SPACE, // flip the y axis
            is_main: true,
        }
    }

    /// Create a new secondary camera, with a perspective projection matrix.
    pub fn secondary_perspective(screen_height: f32, screen_width: f32, z_near: f32, z_far: f32, fov_y_radians: f32) -> Camera {
        let aspect_ratio = screen_width / screen_height;
        let proj = glam::Mat4::perspective_rh(fov_y_radians, aspect_ratio, z_near, z_far);
        Camera {
            properties: CameraTypeProperty::Perspective{
                aspect_ratio, fov_y_radians, z_near, z_far,
            },
            projection_matrix: proj * VK_TO_PROP_SPACE, // flip the y axis
            is_main: false,
        }
    }

    /// Create a new main camera, with an orthographic projection matrix.
    pub fn main_orthographic(screen_height: f32, screen_width: f32, z_near: f32, z_far: f32, view_height: f32) -> Camera {
        let aspect_ratio = screen_width / screen_height;
        let half_view_height = view_height * 0.5;
        let proj = glam::Mat4::orthographic_rh(
            -half_view_height * aspect_ratio, 
            half_view_height * aspect_ratio,
            -half_view_height,
            half_view_height,
            z_near, z_far
        ) * VK_TO_PROP_SPACE;
        Camera {
            properties: CameraTypeProperty::Orthographic{
                aspect_ratio, view_height, z_near, z_far,
            },
            projection_matrix: proj,
            is_main: true,
        }
    }

    /// Create a new secondary camera, with an orthographic projection matrix.
    pub fn secondary_orthographic(screen_height: f32, screen_width: f32, z_near: f32, z_far: f32, view_height: f32) -> Camera {
        let aspect_ratio = screen_width / screen_height;
        let half_view_height = view_height * 0.5;
        let proj = glam::Mat4::orthographic_rh(
            -half_view_height * aspect_ratio, 
            half_view_height * aspect_ratio,
            -half_view_height,
            half_view_height,
            z_near, z_far
        ) * VK_TO_PROP_SPACE;
        Camera {
            properties: CameraTypeProperty::Orthographic{
                aspect_ratio, view_height, z_near, z_far,
            },
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

    pub fn resize(&mut self, new_screen_height: f32, new_screen_width: f32) {
        match &mut self.properties {
            CameraTypeProperty::Perspective{aspect_ratio, fov_y_radians, z_near, z_far} => {
                let new_aspect_ratio = new_screen_width / new_screen_height;
                *aspect_ratio = new_aspect_ratio;
                self.projection_matrix = glam::Mat4::perspective_rh(*fov_y_radians, *aspect_ratio, *z_near, *z_far) * VK_TO_PROP_SPACE;
            },
            CameraTypeProperty::Orthographic{aspect_ratio, view_height, z_near, z_far} => {
                let new_aspect_ratio = new_screen_width / new_screen_height;
                *aspect_ratio = new_aspect_ratio;
                let half_view_height = *view_height * 0.5;
                self.projection_matrix = glam::Mat4::orthographic_rh(
                    -half_view_height * *aspect_ratio, 
                    half_view_height * *aspect_ratio,
                    -half_view_height,
                    half_view_height,
                    *z_near, *z_far
                ) * VK_TO_PROP_SPACE;
            },
        }
    }
}