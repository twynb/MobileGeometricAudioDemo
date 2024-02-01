use crate::scene::{Coordinates, Receiver, Scene, Surface, SurfaceKeyframe};

/// update the `min_coords` and `max_coords` if values from `coords` are smaller/greater than them.
///
/// # Arguments
///
/// * `coords`: The coordinates to compare.
/// * `min_coords`: The scene's minimum coordinates.
/// * `max_coords`: The scene's maximum coordinates.
fn update_maximum_bounds(
    coords: &Coordinates,
    min_coords: &mut Coordinates,
    max_coords: &mut Coordinates,
    radius: Option<f32>
) {
    let radius = radius.unwrap_or(0f32);
    let x = coords.x;
    let y = coords.y;
    let z = coords.z;
    if x - radius < min_coords.x {
        min_coords.x = x - radius
    }
    if y - radius < min_coords.y {
        min_coords.y = y - radius
    }
    if z - radius < min_coords.z {
        min_coords.z = z - radius
    }
    if x + radius > max_coords.x {
        max_coords.x = x + radius
    }
    if y + radius > max_coords.y {
        max_coords.y = y + radius
    }
    if z + radius > max_coords.z {
        max_coords.z = z + radius
    }
}

impl<const N: usize> SurfaceKeyframe<N> {
    /// Get the maximum bounds of the scene where receivers or objects may be.
    /// If a ray travels outside of these bounds without intersecting with anything, it can be discarded.
    /// This could also be used to then define chunks?
    pub fn maximum_bounds(&self) -> (Coordinates, Coordinates) {
        maximum_bounds(&self.coords)
    }
}

impl Scene {
    /// Get the maximum bounds of the scene where receivers or objects may be.
    /// If a ray travels outside of these bounds without intersecting with anything, it can be discarded.
    pub fn maximum_bounds(&self) -> (Coordinates, Coordinates) {
        let mut min_coords = Coordinates {
            x: f32::MAX,
            y: f32::MAX,
            z: f32::MAX,
        };
        let mut max_coords = Coordinates {
            x: f32::MIN,
            y: f32::MIN,
            z: f32::MIN,
        };
        for surface in &self.surfaces {
            match surface {
                Surface::Interpolated(coordinates, _time) => {
                    for coord in coordinates {
                        update_maximum_bounds(coord, &mut min_coords, &mut max_coords, None);
                    }
                }
                Surface::Keyframes(keyframes) => {
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
                update_maximum_bounds(&coordinates, &mut min_coords, &mut max_coords, Some(*radius));
            }
            Receiver::Keyframes(keyframes, radius) => {
                for keyframe in keyframes {
                    update_maximum_bounds(&keyframe.coords, &mut min_coords, &mut max_coords, Some(*radius));
                }
            }
        };

        (min_coords, max_coords)
    }
}

pub fn maximum_bounds(coordinates: &[Coordinates]) -> (Coordinates, Coordinates) {
    let mut min_coords = Coordinates {
        x: f32::MAX,
        y: f32::MAX,
        z: f32::MAX,
    };
    let mut max_coords = Coordinates {
        x: f32::MIN,
        y: f32::MIN,
        z: f32::MIN,
    };
    for coord in coordinates {
        update_maximum_bounds(coord, &mut min_coords, &mut max_coords, None);
    }

    (min_coords, max_coords)
}

#[cfg(test)]
mod tests {
    use crate::scene::{
        CoordinateKeyframe, Coordinates, Emitter, Receiver, Scene, Surface, SurfaceKeyframe,
    };

    fn empty_scene() -> Scene {
        Scene {
            receiver: Receiver::Keyframes(
                vec![CoordinateKeyframe {
                    time: 0,
                    coords: Coordinates {
                        ..Default::default()
                    },
                }],
                0.1f32
            ),
            surfaces: vec![],
            emitter: Emitter::Keyframes(
                vec![CoordinateKeyframe {
                    time: 0,
                    coords: Coordinates {
                        ..Default::default()
                    },
                }],
            ),
        }
    }

    #[test]
    fn maximum_bounds_empty_scene() {
        let scene = empty_scene();
        assert_eq!(
            (
                Coordinates::at(-0.1f32, -0.1f32, -0.1f32),
                Coordinates::at(0.1f32, 0.1f32, 0.1f32)
            ),
            scene.maximum_bounds()
        );
    }

    #[test]
    fn maximum_bounds_moving_receiver_and_ignored_moving_emitter() {
        let scene = Scene {
            receiver: Receiver::Keyframes(
                vec![
                    CoordinateKeyframe {
                        time: 0,
                        coords: Coordinates {
                            ..Default::default()
                        },
                    },
                    CoordinateKeyframe {
                        time: 3,
                        coords: Coordinates::at(20f32, 10f32, 34f32),
                    },
                ],
                0.1f32
            ),
            surfaces: vec![],
            emitter: Emitter::Keyframes(
                vec![
                    CoordinateKeyframe {
                        time: 0,
                        coords: Coordinates {
                            ..Default::default()
                        },
                    },
                    CoordinateKeyframe {
                        time: 3,
                        coords: Coordinates::at(-10f32, -20f32, -50f32),
                    },
                ],
            ),
        };

        assert_eq!(
            (
                Coordinates::at(-0.1f32, -0.1f32, -0.1f32),
                Coordinates::at(20.1f32, 10.1f32, 34.1f32)
            ),
            scene.maximum_bounds()
        );
    }

    #[test]
    fn maximum_bounds_moving_receiver_and_objects_and_ignored_moving_emitter() {
        let scene = Scene {
            receiver: Receiver::Keyframes(
                vec![
                    CoordinateKeyframe {
                        time: 0,
                        coords: Coordinates {
                            ..Default::default()
                        },
                    },
                    CoordinateKeyframe {
                        time: 3,
                        coords: Coordinates::at(20f32, 10f32, 34f32),
                    },
                ],
                0.1
            ),
            surfaces: vec![
                Surface::Keyframes(
                    vec![
                        SurfaceKeyframe {
                            time: 5,
                            coords: [
                                Coordinates::at(-10f32, -20f32, -30f32),
                                Coordinates::at(0f32, 2f32, 16f32),
                                Coordinates::at(0f32, 2f32, 15f32),
                            ],
                        },
                        SurfaceKeyframe {
                            time: 10,
                            coords: [
                                Coordinates::at(3f32, 2f32, 5f32),
                                Coordinates::at(8f32, 10f32, 12f32),
                                Coordinates::at(0f32, 2f32, 16f32),
                            ],
                        },
                    ],
                ),
                Surface::Keyframes(
                    vec![
                        SurfaceKeyframe {
                            time: 5,
                            coords: [
                                Coordinates::at(0f32, 0f32, 0f32),
                                Coordinates::at(0f32, 2f32, 16f32),
                                Coordinates::at(0f32, 4f32, 16f32),
                            ],
                        },
                        SurfaceKeyframe {
                            time: 10,
                            coords: [
                                Coordinates::at(3f32, 2f32, 5f32),
                                Coordinates::at(8f32, 10f32, 12f32),
                                Coordinates::at(0f32, 4f32, 16f32),
                            ],
                        },
                        SurfaceKeyframe {
                            time: 15,
                            coords: [
                                Coordinates::at(0f32, 0f32, 0f32),
                                Coordinates::at(0f32, 2f32, 16f32),
                                Coordinates::at(0f32, 4f32, 16f32),
                            ],
                        },
                    ],
                ),
            ],
            emitter: Emitter::Keyframes(
                vec![
                    CoordinateKeyframe {
                        time: 0,
                        coords: Coordinates {
                            ..Default::default()
                        },
                    },
                    CoordinateKeyframe {
                        time: 3,
                        coords: Coordinates::at(-10f32, -20f32, -50f32),
                    },
                ],
            ),
        };

        assert_eq!(
            (
                Coordinates::at(-10f32, -20f32, -30f32),
                Coordinates::at(20.1f32, 10.1f32, 34.1f32)
            ),
            scene.maximum_bounds()
        );
    }
}
