/// base coordinates pub struct
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Coordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Coordinates {
    fn default() -> Self {
        return Self {
            x: 0f32,
            y: 0f32,
            z: 0f32,
            w: 1f32,
        };
    }
}

/// Keyframe for a single set of coordinates.
#[derive(PartialEq, Debug)]
pub struct CoordinateKeyframe {
    pub time: u32,
    pub coords: Coordinates,
}

/// Sound emitter.
/// `coordinates` should only be Some when this Emitter is returned by the atTime() function TODO
/// `keyframes` is expected to be sorted by keyframe time.
#[derive(PartialEq, Debug)]
pub struct Emitter {
    pub keyframes: Option<Vec<CoordinateKeyframe>>,
    pub index: usize,
    pub coordinates: Option<Coordinates>,
}

/// Sound receiver.
/// `coordinates` should only be Some when this Receiver is returned by the atTime() function TODO
/// `keyframes` is expected to be sorted by keyframe time.
#[derive(PartialEq, Debug)]
pub struct Receiver {
    pub keyframes: Option<Vec<CoordinateKeyframe>>,
    pub index: usize,
    pub coordinates: Option<Coordinates>,
}

/// Keyframe for a set of coordinates for an object.
#[derive(PartialEq, Debug)]
pub struct ObjectKeyframe {
    pub time: u32,
    pub coords: Vec<Coordinates>,
}

/// Object in the scene.
/// `coordinates` should only be Some when this Object is returned by the atTime() function TODO
/// `keyframes` is expected to be sorted by keyframe time.
/// It is expected that all CoordinateKeyframes have the same amount of coordinates.
#[derive(PartialEq, Debug)]
pub struct Object {
    pub keyframes: Option<Vec<ObjectKeyframe>>,
    pub index: usize,
    pub coordinates: Option<Vec<Coordinates>>,
}

#[derive(PartialEq, Debug)]
pub struct Scene {
    pub objects: Vec<Object>,
    pub receiver: Receiver,
    pub emitter: Emitter,
}

impl Scene {
    /// Get the maximum bounds of the scene where receivers or objects may be.
    /// If a ray travels outside of these bounds without intersecting with anything, it can be discarded.
    /// This could also be used to then define chunks?
    fn maximum_bounds(&self) -> (Coordinates, Coordinates) {
        let mut min_coords = Coordinates {
            x: f32::MAX,
            y: f32::MAX,
            z: f32::MAX,
            w: 1f32,
        };
        let mut max_coords = Coordinates {
            x: f32::MIN,
            y: f32::MIN,
            z: f32::MIN,
            w: 1f32,
        };
        for object in &self.objects {
            if object.keyframes.is_none() {
                continue;
            };
            for keyframe in object.keyframes.as_ref().unwrap() {
                for coord in &keyframe.coords {
                    update_maximum_bounds(coord, &mut min_coords, &mut max_coords);
                }
            }
        }
        if self.receiver.keyframes.is_some() {
            for keyframe in self.receiver.keyframes.as_ref().unwrap() {
                update_maximum_bounds(&keyframe.coords, &mut min_coords, &mut max_coords);
            }
        }

        (min_coords, max_coords)
    }
}

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
) {
    let x = coords.x / coords.w;
    let y = coords.y / coords.w;
    let z = coords.z / coords.w;
    if x < min_coords.x {
        min_coords.x = x
    }
    if y < min_coords.y {
        min_coords.y = y
    }
    if z < min_coords.z {
        min_coords.z = z
    }
    if x > max_coords.x {
        max_coords.x = x
    }
    if y > max_coords.y {
        max_coords.y = y
    }
    if z > max_coords.z {
        max_coords.z = z
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CoordinateKeyframe, Coordinates, Emitter, Object, ObjectKeyframe, Receiver, Scene,
    };

    #[test]
    fn maximum_bounds_empty_scene() {
        let scene = Scene {
            receiver: Receiver {
                keyframes: Some(vec![CoordinateKeyframe {
                    time: 0,
                    coords: Coordinates {
                        ..Default::default()
                    },
                }]),
                index: 0,
                coordinates: None,
            },
            objects: vec![],
            emitter: Emitter {
                keyframes: Some(vec![CoordinateKeyframe {
                    time: 0,
                    coords: Coordinates {
                        ..Default::default()
                    },
                }]),
                index: 0,
                coordinates: None,
            },
        };

        assert_eq!(
            (
                Coordinates {
                    x: 0f32,
                    y: 0f32,
                    z: 0f32,
                    w: 1f32
                },
                Coordinates {
                    x: 0f32,
                    y: 0f32,
                    z: 0f32,
                    w: 1f32
                }
            ),
            scene.maximum_bounds()
        );
    }

    #[test]
    fn maximum_bounds_moving_receiver_and_ignored_moving_emitter() {
        let scene = Scene {
            receiver: Receiver {
                keyframes: Some(vec![
                    CoordinateKeyframe {
                        time: 0,
                        coords: Coordinates {
                            ..Default::default()
                        },
                    },
                    CoordinateKeyframe {
                        time: 3,
                        coords: Coordinates {
                            x: 10f32,
                            y: 5f32,
                            z: 17f32,
                            w: 0.5f32,
                        },
                    },
                ]),
                index: 0,
                coordinates: None,
            },
            objects: vec![],
            emitter: Emitter {
                keyframes: Some(vec![
                    CoordinateKeyframe {
                        time: 0,
                        coords: Coordinates {
                            ..Default::default()
                        },
                    },
                    CoordinateKeyframe {
                        time: 3,
                        coords: Coordinates {
                            x: -20f32,
                            y: -40f32,
                            z: -100f32,
                            w: 2f32,
                        },
                    },
                ]),
                index: 0,
                coordinates: None,
            },
        };

        assert_eq!(
            (
                Coordinates {
                    x: 0f32,
                    y: 0f32,
                    z: 0f32,
                    w: 1f32
                },
                Coordinates {
                    x: 20f32,
                    y: 10f32,
                    z: 34f32,
                    w: 1f32
                }
            ),
            scene.maximum_bounds()
        );
    }

    #[test]
    fn maximum_bounds_moving_receiver_and_objects_and_ignored_moving_emitter() {
        let scene = Scene {
            receiver: Receiver {
                keyframes: Some(vec![
                    CoordinateKeyframe {
                        time: 0,
                        coords: Coordinates {
                            ..Default::default()
                        },
                    },
                    CoordinateKeyframe {
                        time: 3,
                        coords: Coordinates {
                            x: 10f32,
                            y: 5f32,
                            z: 17f32,
                            w: 0.5f32,
                        },
                    },
                ]),
                index: 0,
                coordinates: None,
            },
            objects: vec![
                Object {
                    index: 0,
                    coordinates: None,
                    keyframes: Some(vec![
                        ObjectKeyframe {
                            time: 5,
                            coords: vec![
                                Coordinates {
                                    x: -1f32,
                                    y: -2f32,
                                    z: -3f32,
                                    w: 0.1f32,
                                },
                                Coordinates {
                                    x: 0f32,
                                    y: 1f32,
                                    z: 8f32,
                                    w: 0.5f32,
                                },
                            ],
                        },
                        ObjectKeyframe {
                            time: 10,
                            coords: vec![
                                Coordinates {
                                    x: 3f32,
                                    y: 2f32,
                                    z: 5f32,
                                    w: 1f32,
                                },
                                Coordinates {
                                    x: 4f32,
                                    y: 5f32,
                                    z: 6f32,
                                    w: 0.5f32,
                                },
                            ],
                        },
                    ]),
                },
                Object {
                    index: 1,
                    coordinates: None,
                    keyframes: Some(vec![
                        ObjectKeyframe {
                            time: 5,
                            coords: vec![
                                Coordinates {
                                    x: 0f32,
                                    y: 0f32,
                                    z: 0f32,
                                    w: 0.1f32,
                                },
                                Coordinates {
                                    x: 0f32,
                                    y: 1f32,
                                    z: 8f32,
                                    w: 0.5f32,
                                },
                                Coordinates {
                                    x: 0f32,
                                    y: 2f32,
                                    z: 8f32,
                                    w: 0.5f32,
                                },
                            ],
                        },
                        ObjectKeyframe {
                            time: 10,
                            coords: vec![
                                Coordinates {
                                    x: 3f32,
                                    y: 2f32,
                                    z: 5f32,
                                    w: 1f32,
                                },
                                Coordinates {
                                    x: 4f32,
                                    y: 5f32,
                                    z: 6f32,
                                    w: 0.5f32,
                                },
                                Coordinates {
                                    x: 0f32,
                                    y: 2f32,
                                    z: 8f32,
                                    w: 0.5f32,
                                },
                            ],
                        },
                        ObjectKeyframe {
                            time: 15,
                            coords: vec![
                                Coordinates {
                                    x: 0f32,
                                    y: 0f32,
                                    z: 0f32,
                                    w: 0.1f32,
                                },
                                Coordinates {
                                    x: 0f32,
                                    y: 1f32,
                                    z: 8f32,
                                    w: 0.5f32,
                                },
                                Coordinates {
                                    x: 0f32,
                                    y: 2f32,
                                    z: 8f32,
                                    w: 0.5f32,
                                },
                            ],
                        },
                    ]),
                },
            ],
            emitter: Emitter {
                keyframes: Some(vec![
                    CoordinateKeyframe {
                        time: 0,
                        coords: Coordinates {
                            ..Default::default()
                        },
                    },
                    CoordinateKeyframe {
                        time: 3,
                        coords: Coordinates {
                            x: -20f32,
                            y: -40f32,
                            z: -100f32,
                            w: 2f32,
                        },
                    },
                ]),
                index: 0,
                coordinates: None,
            },
        };

        assert_eq!(
            (
                Coordinates {
                    x: -10f32,
                    y: -20f32,
                    z: -30f32,
                    w: 1f32
                },
                Coordinates {
                    x: 20f32,
                    y: 10f32,
                    z: 34f32,
                    w: 1f32
                }
            ),
            scene.maximum_bounds()
        );
    }
}
