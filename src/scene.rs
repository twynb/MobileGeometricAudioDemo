use itertools::Itertools;

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
pub struct ObjectKeyframe<const N: usize> {
    pub time: u32,
    pub coords: [Coordinates; N],
}

impl<const N: usize> ObjectKeyframe<N> {
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
        for coord in &self.coords {
            update_maximum_bounds(coord, &mut min_coords, &mut max_coords);
        }

        (min_coords, max_coords)
    }
}

/// Object in the scene.
/// `coordinates` should only be Some when this Object is returned by the atTime() function TODO
/// `keyframes` is expected to be sorted by keyframe time.
/// It is expected that all CoordinateKeyframes have the same amount of coordinates.
#[derive(PartialEq, Debug)]
pub struct Surface<const N: usize> {
    pub keyframes: Option<Vec<ObjectKeyframe<N>>>,
    pub index: usize,
    pub coordinates: Option<[Coordinates; N]>,
}

#[derive(PartialEq, Debug)]
pub struct Scene {
    pub surfaces: Vec<Surface<4>>, // for now we only work with rectangles
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
        for object in &self.surfaces {
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

    /// Calculate the chunks for this scene.
    ///
    /// The amount of chunks calculated is determined by N - a higher amount will provide more accuracy
    /// when using the chunks (i.e. less needless intersection calculations), but will be more expensive to calculate.
    /// A balance for what amount of chunks is worthwhile needs to be determined via benchmarking.
    ///
    /// For objects and receivers, the chunks they are in are calculated on a per-keyframe-pair basis:
    /// Each keyframe pair (so the first and second, second and third, ...) is iterated over individually, calculating the min/max
    /// coordinates per pair and adding the object to all chunks within those min/max coordinates.
    /// This avoids excessive chunking in cases where, for example, an object moves along an L-shaped path.
    /// 
    /// TODO: Test
    fn chunks<const N: usize>(&self) -> ([[[SceneChunk; N]; N]; N], (f32, f32, f32)) {
        let mut result: [[[SceneChunk; N]; N]; N] = array_init::array_init(|_| {
            array_init::array_init(|_2| {
                array_init::array_init(|_3| SceneChunk {
                    object_indices: vec![],
                    receiver_indices: vec![],
                })
            })
        });
        let mut final_result = result.clone();

        let (min_coords, max_coords) = self.maximum_bounds();
        let x_chunk_size = (max_coords.x - min_coords.x) / N as f32;
        let y_chunk_size = (max_coords.y - min_coords.y) / N as f32;
        let z_chunk_size = (max_coords.z - min_coords.z) / N as f32;

        // calculate gradients between each keyframe
        // take chunks within those gradients
        for object in &self.surfaces {
            if object.keyframes.is_none() {
                continue;
            }
            let keyframes = object.keyframes.as_ref().unwrap();
            let (mut first_keyframe_min, mut first_keyframe_max) = keyframes[0].maximum_bounds();
            for idx in 1..keyframes.len() {
                let (second_keyframe_min, second_keyframe_max) = keyframes[idx].maximum_bounds();
                update_maximum_bounds(
                    &second_keyframe_min,
                    &mut first_keyframe_min,
                    &mut first_keyframe_max,
                );
                update_maximum_bounds(
                    &second_keyframe_max,
                    &mut first_keyframe_min,
                    &mut first_keyframe_max,
                );

                let x_first_chunk =
                    ((first_keyframe_min.x - min_coords.x) / x_chunk_size).floor() as usize;
                let y_first_chunk =
                    ((first_keyframe_min.y - min_coords.y) / y_chunk_size).floor() as usize;
                let z_first_chunk =
                    ((first_keyframe_min.z - min_coords.z) / z_chunk_size).floor() as usize;
                let x_last_chunk =
                    ((first_keyframe_max.x - max_coords.x) / x_chunk_size).floor() as usize;
                let y_last_chunk =
                    ((first_keyframe_max.y - max_coords.y) / y_chunk_size).floor() as usize;
                let z_last_chunk =
                    ((first_keyframe_max.z - max_coords.z) / z_chunk_size).floor() as usize;
                for x in x_first_chunk..=x_last_chunk {
                    for y in y_first_chunk..=y_last_chunk {
                        for z in z_first_chunk..=z_last_chunk {
                            result[x][y][z].object_indices.push(object.index);
                        }
                    }
                }

                first_keyframe_min = second_keyframe_min;
                first_keyframe_max = second_keyframe_max;
            }
        }

        if self.receiver.keyframes.is_some() {
            let keyframes = self.receiver.keyframes.as_ref().unwrap();
            let mut first_keyframe = keyframes[0].coords;
            for idx in 1..keyframes.len() {
                let second_keyframe = keyframes[idx].coords;
                let x_min = first_keyframe.x.min(second_keyframe.x);
                let y_min = first_keyframe.y.min(second_keyframe.y);
                let z_min = first_keyframe.z.min(second_keyframe.z);
                let x_max = first_keyframe.x.max(second_keyframe.x);
                let y_max = first_keyframe.y.max(second_keyframe.y);
                let z_max = first_keyframe.z.max(second_keyframe.z);

                let x_first_chunk = ((x_min - min_coords.x) / x_chunk_size).floor() as usize;
                let y_first_chunk = ((y_min - min_coords.y) / y_chunk_size).floor() as usize;
                let z_first_chunk = ((z_min - min_coords.z) / z_chunk_size).floor() as usize;
                let x_last_chunk = ((x_max - max_coords.x) / x_chunk_size).floor() as usize;
                let y_last_chunk = ((y_max - max_coords.y) / y_chunk_size).floor() as usize;
                let z_last_chunk = ((z_max - max_coords.z) / z_chunk_size).floor() as usize;
                for x in x_first_chunk..=x_last_chunk {
                    for y in y_first_chunk..=y_last_chunk {
                        for z in z_first_chunk..=z_last_chunk {
                            result[x][y][z].receiver_indices.push(self.receiver.index);
                        }
                    }
                }
                first_keyframe = second_keyframe;
            }
        }

        for x in 0..N {
            for y in 0..N {
                for z in 0..N {
                    final_result[x][y][z].object_indices = result[x][y][z]
                        .object_indices
                        .clone()
                        .into_iter()
                        .unique()
                        .collect();
                    final_result[x][y][z].receiver_indices = result[x][y][z]
                        .receiver_indices
                        .clone()
                        .into_iter()
                        .unique()
                        .collect();
                }
            }
        }

        (result, (x_chunk_size, y_chunk_size, z_chunk_size))
    }
}

#[derive(Clone)]
struct SceneChunk {
    object_indices: Vec<usize>,
    receiver_indices: Vec<usize>,
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
        CoordinateKeyframe, Coordinates, Emitter, Surface, ObjectKeyframe, Receiver, Scene,
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
            surfaces: vec![],
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
            surfaces: vec![],
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
            surfaces: vec![
                Surface {
                    index: 0,
                    coordinates: None,
                    keyframes: Some(vec![
                        ObjectKeyframe {
                            time: 5,
                            coords: [
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
                                Coordinates {
                                    x: 0f32,
                                    y: 1f32,
                                    z: 8f32,
                                    w: 0.5f32,
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
                            coords: [
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
                                    y: 1f32,
                                    z: 8f32,
                                    w: 0.5f32,
                                },
                                Coordinates {
                                    x: 0f32,
                                    y: 1f32,
                                    z: 8f32,
                                    w: 0.5f32,
                                },
                            ],
                        },
                    ]),
                },
                Surface {
                    index: 1,
                    coordinates: None,
                    keyframes: Some(vec![
                        ObjectKeyframe {
                            time: 5,
                            coords: [
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
                            coords: [
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
                            coords: [
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

    #[test]
    fn test_chunks_empty_scene() {}
}
