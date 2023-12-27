use crate::scene::{Coordinates, Scene, SurfaceKeyframe};
use itertools::Itertools;

#[derive(Clone)]
struct SceneChunk {
    object_indices: Vec<usize>,
    receiver_indices: Vec<usize>,
}

type Chunks3D<const N: usize> = [[[SceneChunk; N]; N]; N];

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

impl<const N: usize> SurfaceKeyframe<N> {
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
        for surface in &self.surfaces {
            if surface.keyframes.is_none() {
                continue;
            };
            for keyframe in surface.keyframes.as_ref().unwrap() {
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
    fn chunks<const N: usize>(&self) -> (Chunks3D<N>, (f32, f32, f32)) {
        let mut result: Chunks3D<N> = array_init::array_init(|_| {
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
        for surface in &self.surfaces {
            if surface.keyframes.is_none() {
                continue;
            }
            let keyframes = surface.keyframes.as_ref().unwrap();
            let (mut first_keyframe_min, mut first_keyframe_max) = keyframes[0].maximum_bounds();
            for keyframe in keyframes.iter().skip(1) {
                let (second_keyframe_min, second_keyframe_max) = keyframe.maximum_bounds();
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

                let (x_first_chunk, y_first_chunk, z_first_chunk) = coords_to_chunk_index(
                    &first_keyframe_min,
                    &min_coords,
                    x_chunk_size,
                    y_chunk_size,
                    z_chunk_size,
                );
                let (x_last_chunk, y_last_chunk, z_last_chunk) = coords_to_chunk_index(
                    &first_keyframe_max,
                    &max_coords,
                    x_chunk_size,
                    y_chunk_size,
                    z_chunk_size,
                );

                for x in result.iter_mut().take(x_last_chunk + 1).skip(x_first_chunk) {
                    for y in x.iter_mut().take(y_last_chunk + 1).skip(y_first_chunk) {
                        for z in y.iter_mut().take(z_last_chunk + 1).skip(z_first_chunk) {
                            z.object_indices.push(surface.index);
                        }
                    }
                }

                first_keyframe_min = second_keyframe_min;
                first_keyframe_max = second_keyframe_max;
            }
        }

        if self.receiver.keyframes.is_some() {
            let keyframes = self.receiver.keyframes.as_ref().unwrap();
            let mut first_keyframe_coords = keyframes[0].coords;
            for keyframe in keyframes.iter().skip(1) {
                let second_keyframe_coords = keyframe.coords;
                let min = first_keyframe_coords.min_coords(&second_keyframe_coords);
                let max = first_keyframe_coords.max_coords(&second_keyframe_coords);

                let (x_first_chunk, y_first_chunk, z_first_chunk) = coords_to_chunk_index(
                    &min,
                    &min_coords,
                    x_chunk_size,
                    y_chunk_size,
                    z_chunk_size,
                );
                let (x_last_chunk, y_last_chunk, z_last_chunk) = coords_to_chunk_index(
                    &max,
                    &max_coords,
                    x_chunk_size,
                    y_chunk_size,
                    z_chunk_size,
                );

                for x in result.iter_mut().take(x_last_chunk + 1).skip(x_first_chunk) {
                    for y in x.iter_mut().take(y_last_chunk + 1).skip(y_first_chunk) {
                        for z in y.iter_mut().take(z_last_chunk + 1).skip(z_first_chunk) {
                            z.receiver_indices.push(self.receiver.index);
                        }
                    }
                }
                first_keyframe_coords = second_keyframe_coords;
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

fn coords_to_chunk_index(
    coords: &Coordinates,
    min_coords: &Coordinates,
    x_chunk_size: f32,
    y_chunk_size: f32,
    z_chunk_size: f32,
) -> (usize, usize, usize) {
    (
        ((coords.x - min_coords.x) / x_chunk_size).floor() as usize,
        ((coords.y - min_coords.y) / y_chunk_size).floor() as usize,
        ((coords.z - min_coords.z) / z_chunk_size).floor() as usize,
    )
}

#[cfg(test)]
mod tests {
    use crate::scene::{
        CoordinateKeyframe, Coordinates, Emitter, Receiver, Scene, Surface, SurfaceKeyframe,
    };

    fn empty_scene() -> Scene {
        Scene {
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
        }
    }

    #[test]
    fn maximum_bounds_empty_scene() {
        let scene = empty_scene();
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
                        SurfaceKeyframe {
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
                        SurfaceKeyframe {
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
                        SurfaceKeyframe {
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
                        SurfaceKeyframe {
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
                        SurfaceKeyframe {
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
    fn test_chunks_empty_scene() {
        let _scene = empty_scene();
    }
}
