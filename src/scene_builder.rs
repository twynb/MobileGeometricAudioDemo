use crate::scene::{Coordinates, Surface};

pub fn static_cube(bottom_left: Coordinates, top_right: Coordinates) -> Vec<Surface<3>> {
    vec![
        // left
        Surface::Interpolated(
            [
                Coordinates::at(bottom_left.x, bottom_left.y, bottom_left.z),
                Coordinates::at(bottom_left.x, bottom_left.y, top_right.z),
                Coordinates::at(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
        ),
        Surface::Interpolated(
            [
                Coordinates::at(bottom_left.x, top_right.y, top_right.z),
                Coordinates::at(bottom_left.x, bottom_left.y, top_right.z),
                Coordinates::at(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
        ),
        // front
        Surface::Interpolated(
            [
                Coordinates::at(bottom_left.x, bottom_left.y, bottom_left.z),
                Coordinates::at(top_right.x, bottom_left.y, bottom_left.z),
                Coordinates::at(bottom_left.x, bottom_left.y, top_right.z),
            ],
            0,
        ),
        Surface::Interpolated(
            [
                Coordinates::at(top_right.x, bottom_left.y, top_right.z),
                Coordinates::at(top_right.x, bottom_left.y, bottom_left.z),
                Coordinates::at(bottom_left.x, bottom_left.y, top_right.z),
            ],
            0,
        ),
        // right
        Surface::Interpolated(
            [
                Coordinates::at(top_right.x, bottom_left.y, bottom_left.z),
                Coordinates::at(top_right.x, bottom_left.y, top_right.z),
                Coordinates::at(top_right.x, top_right.y, bottom_left.z),
            ],
            0,
        ),
        Surface::Interpolated(
            [
                Coordinates::at(top_right.x, top_right.y, top_right.z),
                Coordinates::at(top_right.x, bottom_left.y, top_right.z),
                Coordinates::at(top_right.x, top_right.y, bottom_left.z),
            ],
            0,
        ),
        // back
        Surface::Interpolated(
            [
                Coordinates::at(bottom_left.x, top_right.y, bottom_left.z),
                Coordinates::at(top_right.x, top_right.y, bottom_left.z),
                Coordinates::at(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
        ),
        Surface::Interpolated(
            [
                Coordinates::at(top_right.x, top_right.y, top_right.z),
                Coordinates::at(top_right.x, top_right.y, bottom_left.z),
                Coordinates::at(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
        ),
        // bottom
        Surface::Interpolated(
            [
                Coordinates::at(bottom_left.x, bottom_left.y, bottom_left.z),
                Coordinates::at(top_right.x, bottom_left.y, bottom_left.z),
                Coordinates::at(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
        ),
        Surface::Interpolated(
            [
                Coordinates::at(top_right.x, top_right.y, bottom_left.z),
                Coordinates::at(top_right.x, bottom_left.y, bottom_left.z),
                Coordinates::at(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
        ),
        // top
        Surface::Interpolated(
            [
                Coordinates::at(bottom_left.x, bottom_left.y, top_right.z),
                Coordinates::at(top_right.x, bottom_left.y, top_right.z),
                Coordinates::at(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
        ),
        Surface::Interpolated(
            [
                Coordinates::at(top_right.x, top_right.y, top_right.z),
                Coordinates::at(top_right.x, bottom_left.y, top_right.z),
                Coordinates::at(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
        ),
    ]
}
