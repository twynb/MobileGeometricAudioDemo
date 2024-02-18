use nalgebra::Vector3;

use crate::scene::{Emitter, Receiver, Scene, Surface, SurfaceKeyframe};

pub trait MaximumBounds {
    /// Get the maximum bounds of the element(s) described by this object.
    fn maximum_bounds(&self) -> (Vector3<f32>, Vector3<f32>);
}

/// update the `min_coords` and `max_coords` if values from `coords` are smaller/greater than them.
///
/// # Arguments
///
/// * `coords`: The coordinates to compare.
/// * `min_coords`: The scene's minimum coordinates.
/// * `max_coords`: The scene's maximum coordinates.
fn update_maximum_bounds(
    coords: &Vector3<f32>,
    min_coords: &mut Vector3<f32>,
    max_coords: &mut Vector3<f32>,
    radius: Option<f32>,
) {
    let radius = radius.unwrap_or(0f32);
    let x = coords.x;
    let y = coords.y;
    let z = coords.z;
    if x - radius < min_coords.x {
        min_coords.x = x - radius;
    }
    if y - radius < min_coords.y {
        min_coords.y = y - radius;
    }
    if z - radius < min_coords.z {
        min_coords.z = z - radius;
    }
    if x + radius > max_coords.x {
        max_coords.x = x + radius;
    }
    if y + radius > max_coords.y {
        max_coords.y = y + radius;
    }
    if z + radius > max_coords.z {
        max_coords.z = z + radius;
    }
}

impl<const N: usize> MaximumBounds for SurfaceKeyframe<N> {
    fn maximum_bounds(&self) -> (Vector3<f32>, Vector3<f32>) {
        maximum_bounds(&self.coords)
    }
}

impl MaximumBounds for Scene {
    fn maximum_bounds(&self) -> (Vector3<f32>, Vector3<f32>) {
        let mut min_coords: Vector3<f32> = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max_coords: Vector3<f32> = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
        for surface in &self.surfaces {
            match surface {
                Surface::Interpolated(coordinates, _time, _material) => {
                    for coord in coordinates {
                        update_maximum_bounds(coord, &mut min_coords, &mut max_coords, None);
                    }
                }
                Surface::Keyframes(keyframes, _material) => {
                    for keyframe in keyframes {
                        for coord in &keyframe.coords {
                            update_maximum_bounds(coord, &mut min_coords, &mut max_coords, None);
                        }
                    }
                }
            };
        }
        match &self.receiver {
            Receiver::Interpolated(coordinates, radius, _time) => {
                update_maximum_bounds(coordinates, &mut min_coords, &mut max_coords, Some(*radius));
            }
            Receiver::Keyframes(keyframes, radius) => {
                for keyframe in keyframes {
                    update_maximum_bounds(
                        &keyframe.coords,
                        &mut min_coords,
                        &mut max_coords,
                        Some(*radius),
                    );
                }
            }
        };
        match &self.emitter {
            Emitter::Interpolated(coordinates, _time, _emission_type) => {
                update_maximum_bounds(coordinates, &mut min_coords, &mut max_coords, Some(0.1f32));
            }
            Emitter::Keyframes(keyframes, _emission_type) => {
                for keyframe in keyframes {
                    update_maximum_bounds(
                        &keyframe.coords,
                        &mut min_coords,
                        &mut max_coords,
                        Some(0.1f32),
                    );
                }
            }
        };

        (min_coords, max_coords)
    }
}

/// Get the maximum bounds of the object described by the given coordinates.
pub fn maximum_bounds(coordinates: &[Vector3<f32>]) -> (Vector3<f32>, Vector3<f32>) {
    let mut min_coords: Vector3<f32> = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max_coords: Vector3<f32> = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
    for coord in coordinates {
        update_maximum_bounds(coord, &mut min_coords, &mut max_coords, None);
    }

    (min_coords, max_coords)
}

#[cfg(test)]
mod tests {
    use nalgebra::Vector3;

    use super::MaximumBounds;
    use crate::{
        bounce::EmissionType,
        materials::MATERIAL_CONCRETE_WALL,
        scene::{CoordinateKeyframe, Emitter, Receiver, Scene, Surface, SurfaceKeyframe},
    };

    fn empty_scene() -> Scene {
        Scene {
            receiver: Receiver::Keyframes(
                vec![CoordinateKeyframe {
                    time: 0,
                    coords: Vector3::new(0f32, 0f32, 0f32),
                }],
                0.1f32,
            ),
            surfaces: vec![],
            emitter: Emitter::Keyframes(
                vec![CoordinateKeyframe {
                    time: 0,
                    coords: Vector3::new(0f32, 0f32, 0f32),
                }],
                EmissionType::Random,
            ),
        }
    }

    #[test]
    fn maximum_bounds_empty_scene() {
        let scene = empty_scene();
        assert_eq!(
            (
                Vector3::new(-0.1f32, -0.1f32, -0.1f32),
                Vector3::new(0.1f32, 0.1f32, 0.1f32)
            ),
            scene.maximum_bounds()
        );
    }

    #[test]
    fn maximum_bounds_moving_receiver_and_moving_emitter() {
        let scene = Scene {
            receiver: Receiver::Keyframes(
                vec![
                    CoordinateKeyframe {
                        time: 0,
                        coords: Vector3::new(0f32, 0f32, 0f32),
                    },
                    CoordinateKeyframe {
                        time: 3,
                        coords: Vector3::new(20f32, 10f32, 34f32),
                    },
                ],
                0.1f32,
            ),
            surfaces: vec![],
            emitter: Emitter::Keyframes(
                vec![
                    CoordinateKeyframe {
                        time: 0,
                        coords: Vector3::new(0f32, 0f32, 0f32),
                    },
                    CoordinateKeyframe {
                        time: 3,
                        coords: Vector3::new(-10f32, -20f32, -50f32),
                    },
                ],
                EmissionType::Random,
            ),
        };

        assert_eq!(
            (
                Vector3::new(-10.1f32, -20.1f32, -50.1f32),
                Vector3::new(20.1f32, 10.1f32, 34.1f32)
            ),
            scene.maximum_bounds()
        );
    }

    #[test]
    fn maximum_bounds_moving_receiver_and_objects_and_moving_emitter() {
        let scene = Scene {
            receiver: Receiver::Keyframes(
                vec![
                    CoordinateKeyframe {
                        time: 0,
                        coords: Vector3::new(0f32, 0f32, 0f32),
                    },
                    CoordinateKeyframe {
                        time: 3,
                        coords: Vector3::new(20f32, 10f32, 34f32),
                    },
                ],
                0.1,
            ),
            surfaces: vec![
                Surface::Keyframes(
                    vec![
                        SurfaceKeyframe {
                            time: 5,
                            coords: [
                                Vector3::new(-10f32, -20f32, -30f32),
                                Vector3::new(0f32, 2f32, 16f32),
                                Vector3::new(0f32, 2f32, 15f32),
                            ],
                        },
                        SurfaceKeyframe {
                            time: 10,
                            coords: [
                                Vector3::new(3f32, 2f32, 5f32),
                                Vector3::new(8f32, 10f32, 12f32),
                                Vector3::new(0f32, 2f32, 16f32),
                            ],
                        },
                    ],
                    MATERIAL_CONCRETE_WALL,
                ),
                Surface::Keyframes(
                    vec![
                        SurfaceKeyframe {
                            time: 5,
                            coords: [
                                Vector3::new(0f32, 0f32, 0f32),
                                Vector3::new(0f32, 2f32, 16f32),
                                Vector3::new(0f32, 4f32, 16f32),
                            ],
                        },
                        SurfaceKeyframe {
                            time: 10,
                            coords: [
                                Vector3::new(3f32, 2f32, 5f32),
                                Vector3::new(8f32, 10f32, 12f32),
                                Vector3::new(0f32, 4f32, 16f32),
                            ],
                        },
                        SurfaceKeyframe {
                            time: 15,
                            coords: [
                                Vector3::new(0f32, 0f32, 0f32),
                                Vector3::new(0f32, 2f32, 16f32),
                                Vector3::new(0f32, 4f32, 16f32),
                            ],
                        },
                    ],
                    MATERIAL_CONCRETE_WALL,
                ),
            ],
            emitter: Emitter::Keyframes(vec![
                CoordinateKeyframe {
                    time: 0,
                    coords: Vector3::new(0f32, 0f32, 0f32),
                },
                CoordinateKeyframe {
                    time: 3,
                    coords: Vector3::new(-10f32, -20f32, -50f32),
                },
            ], EmissionType::Random),
        };

        assert_eq!(
            (
                Vector3::new(-10.1f32, -20.1f32, -50.1f32),
                Vector3::new(20.1f32, 10.1f32, 34.1f32)
            ),
            scene.maximum_bounds()
        );
    }
}
