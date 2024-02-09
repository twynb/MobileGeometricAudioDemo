use nalgebra::Vector3;

use crate::scene::Surface;

#[must_use]
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
        ),
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
        ),
        // front
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, bottom_left.y, bottom_left.z),
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
            ],
            0,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
            ],
            0,
        ),
        // right
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
            ],
            0,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, top_right.y, top_right.z),
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
            ],
            0,
        ),
        // back
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, top_right.y, top_right.z),
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
        ),
        // bottom
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, bottom_left.y, bottom_left.z),
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
        ),
        // top
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, top_right.y, top_right.z),
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
        ),
    ]
}
