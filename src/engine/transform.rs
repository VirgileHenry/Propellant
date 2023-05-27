use std::cell::Cell;



pub struct Transform {
    position: glam::Vec3,
    rotation: glam::Quat,
    scale: glam::Vec3,
    world_pos: Cell<Option<glam::Mat4>>,
}

impl Transform {
    pub fn origin() -> Transform {
        Transform {
            position: glam::Vec3::new(0.0, 0.0, 0.0),
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::new(1.0, 1.0, 1.0),
            world_pos: Cell::new(Some(glam::Mat4::IDENTITY)),
        }
    }

    fn invalidate_world_pos(&mut self) {
        self.world_pos.set(None);
    }

    pub fn world_pos(&self) -> glam::Mat4 {
        match self.world_pos.get() {
            Some(world_pos) => world_pos,
            None => {
                // if we have been invalidated, recompute world pos, set it and return it.
                let world_pos = glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
                self.world_pos.set(Some(world_pos));
                world_pos
            }
        }
    } 

    pub fn translated(mut self, translation: glam::Vec3) -> Transform {
        self.position += translation;
        self.invalidate_world_pos();
        self
    }

}


