use std::cell::Cell;
use tree_box::TreeBox;


pub struct Transform {
    core: TreeBox<TransformCore>,
}

struct TransformCore {
    position: glam::Vec3,
    rotation: glam::Quat,
    scale: glam::Vec3,
    world_pos: Cell<Option<glam::Mat4>>,
}

impl TransformCore {
    pub fn origin() -> TransformCore {
        TransformCore {
            position: glam::Vec3::new(0.0, 0.0, 0.0),
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::new(1.0, 1.0, 1.0),
            world_pos: Cell::new(None),
        }
    }

    pub fn world_pos(&self) -> Option<glam::Mat4> {
        self.world_pos.get()
    }

    pub fn invalidate_world_pos(&self) {
        self.world_pos.set(None);
    }

    pub fn recompute(
        &self,
        parent_world_pos: Option<glam::Mat4>,
    ) -> glam::Mat4 {
        let world_pos = match parent_world_pos {
            Some(parent_wp) => parent_wp * glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position),
            None => glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position),
        };
        self.world_pos.set(Some(world_pos));
        world_pos
    }
}


impl Transform {
    /// Create a transform at the origin of the world.
    pub fn origin() -> Transform {
        Transform {
            core: TreeBox::from(TransformCore::origin()),
        }
    }

    /// Flag that our world position is no longer valid.
    fn invalidate_world_pos(&mut self) {
        self.core.mutate(|tf| tf.invalidate_world_pos());
        self.core.mutate_children_rec(|tf| tf.invalidate_world_pos());
    }

    /// Get the world position. If it has been invalidate, it will be recomputed. 
    /// This is why we have a cell, to recompute world pos only when needed while still only "reading" it.
    pub fn world_pos(&self) -> glam::Mat4 {
        // rec get on the parent:
        // parent is none -> identity,
        // else -> parent world pos
        self.core.get_parent_rec(
            |tf| tf.world_pos(),
            |tf, parent_world_pos| tf.recompute(parent_world_pos),
        )
    }

    // ==================== Builder setter ====================
    /// Sets the initial position of the transform.
    pub fn translated(mut self, translation: glam::Vec3) -> Transform {
        self.core.mutate(|tf| tf.position = translation);
        self.invalidate_world_pos();
        self
    }

    /// Sets the initial scale of the transform.
    pub fn scaled(mut self, scale: glam::Vec3) -> Transform {
        self.core.mutate(|tf| tf.scale = scale);
        self.invalidate_world_pos();
        self
    }

    /// Sets the initial rotation of the transform.
    pub fn rotated(mut self, rotation: glam::Quat) -> Transform {
        self.core.mutate(|tf| tf.rotation = rotation);
        self.invalidate_world_pos();
        self
    }

    pub fn child_of(mut self, parent: Option<&Transform>) -> Transform {
        self.core.set_parent(parent.map(|v| &v.core));
        self
    }

    // ==================== Getters ====================
    /// Returns the position of the transform.
    pub fn position(&self) -> glam::Vec3 {
        self.core.get(|tf| tf.position)
    }

    /// Returns the position of the transform.
    pub fn rotation(&self) -> glam::Quat {
        self.core.get(|tf| tf.rotation)
    }

    /// Returns the scale of the transform.
    /// This isn't called "scale" because it is already the name of the scalong func.
    pub fn get_scale(&self) -> glam::Vec3 {
        self.core.get(|tf| tf.scale)
    }

    // ==================== Setters ====================
    /// Sets the position of the transform.
    pub fn set_position(&mut self, position: glam::Vec3) {
        self.core.mutate(|tf| tf.position = position);
        self.invalidate_world_pos();
    }

    /// Sets the rotation of the transform.
    pub fn set_rotation(&mut self, rotation: glam::Quat) {
        self.core.mutate(|tf| tf.rotation = rotation);
        self.invalidate_world_pos();
    }

    /// Sets the scale of the transform.
    pub fn set_scale(&mut self, scale: glam::Vec3) {
        self.core.mutate(|tf| tf.scale = scale);
        self.invalidate_world_pos();
    }

    // ==================== Operations ====================
    /// Translate the transform by a given vector.
    pub fn translate(&mut self, translation: glam::Vec3) {
        self.core.mutate(|tf| tf.position += translation);
        self.invalidate_world_pos();
    }

    /// Rotate the transform by a given quaternion.
    pub fn rotate(&mut self, rotation: glam::Quat) {
        self.core.mutate(|tf| tf.rotation *= rotation);
        self.invalidate_world_pos();
    }

    /// Scale the transform by a given vector.
    pub fn scale(&mut self, scale: glam::Vec3) {
        self.core.mutate(|tf| tf.scale *= scale);
        self.invalidate_world_pos();
    }

}
