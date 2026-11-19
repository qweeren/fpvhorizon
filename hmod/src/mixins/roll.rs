fn fpvdrone_roll(value: glam::Mat4) -> glam::Mat4 {
    value * glam::Mat4::from_rotation_z(-mod_fpvdrone::camera_roll())
}
