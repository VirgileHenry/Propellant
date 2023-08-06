use std::cell::Cell;


#[derive(Debug, Clone)]
pub struct Transform {
    position: glam::Vec3,
    rotation: glam::Quat,
    scale: glam::Vec3,
    world_pos: Cell<Option<glam::Mat4>>,
}

impl Transform {
    /// Create a transform at the origin of the world.
    pub fn origin() -> Transform {
        Transform {
            position: glam::Vec3::new(0.0, 0.0, 0.0),
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::new(1.0, 1.0, 1.0),
            world_pos: Cell::new(Some(glam::Mat4::IDENTITY)),
        }
    }

    /// Flag that our world position is no longer valid.
    fn invalidate_world_pos(&mut self) {
        self.world_pos.set(None);
    }

    /// Get the world position. If it has been invalidate, it will be recomputed. 
    /// This is why we have a cell, to recompute world pos only when needed while still only "reading" it.
    pub fn world_pos(&self) -> glam::Mat4 {
        match self.world_pos.get() {
            Some(world_pos) => world_pos,
            None => {
                // if we have been invalidated, recompute world pos, set it and return it.
                // the order is super weird here, but experimentally it does not work the other way around.
                let world_pos = glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
                self.world_pos.set(Some(world_pos));
                world_pos
            }
        }
    } 

    // ==================== Builder setter ====================
    /// Sets the initial position of the transform.
    pub fn translated(mut self, translation: glam::Vec3) -> Transform {
        self.position += translation;
        self.invalidate_world_pos();
        self
    }

    /// Sets the initial scale of the transform.
    pub fn scaled(mut self, scale: glam::Vec3) -> Transform {
        self.scale *= scale;
        self.invalidate_world_pos();
        self
    }

    /// Sets the initial rotation of the transform.
    pub fn rotated(mut self, rotation: glam::Quat) -> Transform {
        self.rotation *= rotation;
        self.invalidate_world_pos();
        self
    }

    // ==================== Getters ====================
    /// Returns the position of the transform.
    pub fn position(&self) -> glam::Vec3 {
        self.position
    }

    /// Returns the position of the transform.
    pub fn rotation(&self) -> glam::Quat {
        self.rotation
    }

    /// Returns the scale of the transform.
    /// This isn't called "scale" because it is already the name of the scalong func.
    pub fn get_scale(&self) -> glam::Vec3 {
        self.scale
    }

    // ==================== Setters ====================
    /// Sets the position of the transform.
    pub fn set_position(&mut self, position: glam::Vec3) {
        self.position = position;
        self.invalidate_world_pos();
    }

    /// Sets the rotation of the transform.
    pub fn set_rotation(&mut self, rotation: glam::Quat) {
        self.rotation = rotation;
        self.invalidate_world_pos();
    }

    /// Sets the scale of the transform.
    pub fn set_scale(&mut self, scale: glam::Vec3) {
        self.scale = scale;
        self.invalidate_world_pos();
    }

    pub unsafe fn set_world_matrix(&mut self, world_matrix: glam::Mat4) {
        self.world_pos.set(Some(world_matrix));
    }

    // ==================== Operations ====================
    /// Translate the transform by a given vector.
    pub fn translate(&mut self, translation: glam::Vec3) {
        self.position += translation;
        self.invalidate_world_pos();
    }

    /// Rotate the transform by a given quaternion.
    pub fn rotate(&mut self, rotation: glam::Quat) {
        self.rotation *= rotation;
        self.invalidate_world_pos();
    }

    /// Scale the transform by a given vector.
    pub fn scale(&mut self, scale: glam::Vec3) {
        self.scale *= scale;
        self.invalidate_world_pos();
    }

}
