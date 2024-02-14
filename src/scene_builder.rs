use nalgebra::Vector3;

use crate::{materials::MATERIAL_CONCRETE_WALL, scene::Surface};

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn static_cube(bottom_left: Vector3<f32>, top_right: Vector3<f32>) -> Vec<Surface<3>> {
    vec![
        // left
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
            MATERIAL_CONCRETE_WALL,
        ),
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
            MATERIAL_CONCRETE_WALL,
        ),
        // front
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, bottom_left.y, bottom_left.z),
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
            ],
            0,
            MATERIAL_CONCRETE_WALL,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
            ],
            0,
            MATERIAL_CONCRETE_WALL,
        ),
        // right
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
            ],
            0,
            MATERIAL_CONCRETE_WALL,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, top_right.y, top_right.z),
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
            ],
            0,
            MATERIAL_CONCRETE_WALL,
        ),
        // back
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
            MATERIAL_CONCRETE_WALL,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, top_right.y, top_right.z),
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
            MATERIAL_CONCRETE_WALL,
        ),
        // bottom
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, bottom_left.y, bottom_left.z),
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
            MATERIAL_CONCRETE_WALL,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
            MATERIAL_CONCRETE_WALL,
        ),
        // top
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
            MATERIAL_CONCRETE_WALL,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, top_right.y, top_right.z),
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
            MATERIAL_CONCRETE_WALL,
        ),
    ]
}
