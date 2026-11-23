fn fpvhorizon_roll(value: glam::Mat4) -> glam::Mat4 {
    value * glam::Mat4::from_rotation_z(-mod_fpvhorizon::camera_roll())
}
