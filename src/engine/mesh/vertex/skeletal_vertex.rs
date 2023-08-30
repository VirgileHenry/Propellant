
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SkeletalVertex {
    position: glam::Vec3,
    normal: glam::Vec3,
    uv: glam::Vec2,
    joint_ids: [u32; 4],
    joint_weights: [f32; 4],
}