use crate::scene::{CoordinateKeyframe, Coordinates, Emitter, Surface, ObjectKeyframe, Receiver};

pub trait Interpolation {
    /// Get a version of this object at the given time.
    /// If the object already has coordinates rather than keyframes, returns a copy of the object.
    /// If the time matches up with a keyframe, use that keyframe's coordinates.
    /// If the time is before the first keyframe's time, the first keyframe is used.
    /// If the time is after the last keyframe's time, the last keyframe is used.
    /// Otherwise, interpolate between the two adjacent keyframes.
    ///
    /// # Arguments
    /// * `time`: The time to calculate.
    fn at_time(&self, time: u32) -> Self;
}

/// Interpolate between the coordinates and return a vector of interpolated coordinates.
/// This assumes `coords1` and `coords2` are of equal length.
///
/// # Arguments
/// * `coords1`: The first set of coordinates to interpolate between.
/// * `coords2`: The second set of coordinates to interpolate between.
/// * `interp_position`: The interpolation position. Should be between 0 (only use first coordinate) and 1 (only second coordinate).
///
/// # Panics
/// * If `coords1` is longer than `coords2`
fn interpolate_coordinate_array<const N: usize>(
    coords1: &[Coordinates],
    coords2: &[Coordinates],
    interp_position: f32,
) -> [Coordinates; N] {
    let mut result = [Coordinates {..Default::default()}; N];

    for (idx, coord) in coords1.iter().enumerate() {
        result[idx] = interpolate_coordinates(
            coord,
            &coords2[idx],
            interp_position,
        )
    }

    result
}

/// Interpolate between the coordinates and return the result.
///
/// # Arguments
/// * `coords1`: The first set of coordinates to interpolate between.
/// * `coords2`: The second set of coordinates to interpolate between.
/// * `interp_position`: The interpolation position. Should be between 0 (only use first coordinate) and 1 (only second coordinate).
fn interpolate_coordinates(
    coords1: &Coordinates,
    coords2: &Coordinates,
    interp_position: f32,
) -> Coordinates {
    Coordinates {
        x: interpolate_single_coordinate(
            coords1.x,
            coords1.w,
            coords2.x,
            coords2.w,
            interp_position,
        ),
        y: interpolate_single_coordinate(
            coords1.y,
            coords1.w,
            coords2.y,
            coords2.w,
            interp_position,
        ),
        z: interpolate_single_coordinate(
            coords1.z,
            coords1.w,
            coords2.z,
            coords2.w,
            interp_position,
        ),
        w: 1f32,
    }
}

/// Get the interpolated value, taking the w value into account.
///
/// # Arguments
/// * `coord1`: The first coordinate.
/// * `w1`: `w` for the first coordinate.
/// * `coord2`: The second coordinate.
/// * `w2`: `w` for the second coordinate.
/// * `interp_position`: The interpolation position. Should be between 0 (only use first coordinate) and 1 (only second coordinate).
#[inline(always)]
fn interpolate_single_coordinate(
    mut coord1: f32,
    w1: f32,
    mut coord2: f32,
    w2: f32,
    interp_position: f32,
) -> f32 {
    coord1 /= w1;
    coord2 /= w2;

    coord1 * interp_position + coord2 * (1f32 - interp_position)
}

/// Calculate the interpolated coordinate at the given time.
/// If the time matches up with a keyframe, use that keyframe's coordinates.
/// If the time is before the first keyframe's time, the first keyframe is used.
/// If the time is after the last keyframe's time, the last keyframe is used.
/// Otherwise, interpolate between the two adjacent keyframes.
///
/// # Arguments
/// * `keyframes`: The keyframes to interpolate between. Must be sorted by time.
/// * `time`: The time.
fn interpolate_coordinate_keyframes(keyframes: &Vec<CoordinateKeyframe>, time: u32) -> Coordinates {
    // return out early if we're after the last keyframe anyway
    if time >= keyframes[keyframes.len() - 1].time {
        return keyframes[keyframes.len() - 1].coords;
    }

    for i in 0..keyframes.len() {
        if time <= keyframes[i].time {
            return keyframes[i].coords;
        }
        if time >= keyframes[i].time && time < keyframes[i + 1].time {
            let interp_position =
                calculate_interp_position(keyframes[i].time, keyframes[i + 1].time, time);
            return interpolate_coordinates(
                &keyframes[i].coords,
                &keyframes[i + 1].coords,
                interp_position,
            );
        }
    }
    // unable to happen
    panic!("Error in interpolation - this should not happen.");
}

/// Calculate the interpolated coordinate at the given time.
/// If the time matches up with a keyframe, use that keyframe's coordinates.
/// If the time is before the first keyframe's time, the first keyframe is used.
/// If the time is after the last keyframe's time, the last keyframe is used.
/// Otherwise, interpolate between the two adjacent keyframes.
///
/// # Arguments
/// * `keyframes`: The keyframes to interpolate between. Must be sorted by time.
/// * `time`: The time.
fn interpolate_object_keyframes<const N: usize>(keyframes: &Vec<ObjectKeyframe<N>>, time: u32) -> [Coordinates; N] {
    // return out early if we're after the last keyframe, otherwise we'd need to iterate over all the keyframes first
    if time >= keyframes[keyframes.len() - 1].time {
        return keyframes[keyframes.len() - 1].coords;
    }

    for i in 0..keyframes.len() {
        if time <= keyframes[i].time {
            return keyframes[i].coords;
        }
        if time >= keyframes[i].time && time < keyframes[i + 1].time {
            let interp_position =
                calculate_interp_position(keyframes[i].time, keyframes[i + 1].time, time);
            return interpolate_coordinate_array(
                &keyframes[i].coords,
                &keyframes[i + 1].coords,
                interp_position,
            );
        }
    }
    // unable to happen
    panic!("Error in interpolation - this should not happen.");
}

/// Calculate the interpolation position, i.e. how much of the keyframe at `first_time`
/// is still left in the coordinates at `time`.
/// This assumes that `first_time` <= `time` <= `second_time`
///
/// # Arguments
/// * `first_time`: Time of the first keyframe.
/// * `second_time`: Time of the second keyframe.
/// * `time`: The current time.
fn calculate_interp_position(first_time: u32, second_time: u32, time: u32) -> f32 {
    ((second_time - time) as f32) / ((second_time - first_time) as f32)
}

impl Interpolation for Emitter {
    fn at_time(&self, time: u32) -> Self {
        // this shouldn't happen
        if self.keyframes.is_none() {
            return Self {
                keyframes: None,
                index: self.index,
                coordinates: self.coordinates,
            };
        }

        let keyframes = self.keyframes.as_ref().unwrap();
        Self {
            keyframes: None,
            index: self.index,
            coordinates: Some(interpolate_coordinate_keyframes(keyframes, time)),
        }
    }
}

impl Interpolation for Receiver {
    fn at_time(&self, time: u32) -> Self {
        // this shouldn't happen
        if self.keyframes.is_none() {
            return Self {
                keyframes: None,
                index: self.index,
                coordinates: self.coordinates,
            };
        }

        let keyframes = self.keyframes.as_ref().unwrap();
        Self {
            keyframes: None,
            index: self.index,
            coordinates: Some(interpolate_coordinate_keyframes(keyframes, time)),
        }
    }
}

impl<const N: usize> Interpolation for Surface<N> {
    fn at_time(&self, time: u32) -> Self {
        // this shouldn't happen
        if self.keyframes.is_none() {
            return Self {
                keyframes: None,
                index: self.index,
                coordinates: self.coordinates,
            };
        }

        let keyframes = self.keyframes.as_ref().unwrap();
        Self {
            keyframes: None,
            index: self.index,
            coordinates: Some(interpolate_object_keyframes(keyframes, time)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scene::{
        CoordinateKeyframe, Coordinates, Emitter, Surface, ObjectKeyframe, Receiver,
    };

    // TODO tests: at_time() für Object

    use super::{
        calculate_interp_position, interpolate_coordinate_keyframes, interpolate_coordinates,
        interpolate_object_keyframes, interpolate_single_coordinate, Interpolation,
    };

    #[test]
    fn interpolate_object() {
        let object = Surface {
            index: 0,
            coordinates: None,
            keyframes: Some(vec![
                ObjectKeyframe {
                    time: 5,
                    coords: [
                        Coordinates {
                            x: 1f32,
                            y: 2f32,
                            z: 3f32,
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
                    coords: [
                        Coordinates {
                            x: 3f32,
                            y: 2f32,
                            z: 5f32,
                            w: 0.1f32,
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
        };
        let result = object.at_time(7);
        assert_eq!(
            Surface {
                index: 0,
                keyframes: None,
                coordinates: Some([
                    Coordinates {
                        x: 18f32,
                        y: 20f32,
                        z: 38f32,
                        w: 1f32,
                    },
                    Coordinates {
                        x: 3.1999998f32,
                        y: 5.2f32,
                        z: 14.4f32,
                        w: 1f32,
                    },
                ])
            },
            result
        )
    }

    #[test]
    fn interpolate_receiver() {
        let receiver = Receiver {
            index: 0,
            coordinates: None,
            keyframes: Some(vec![
                CoordinateKeyframe {
                    time: 5,
                    coords: Coordinates {
                        x: 3f32,
                        y: 4f32,
                        z: 0f32,
                        w: 0.1f32,
                    },
                },
                CoordinateKeyframe {
                    time: 10,
                    coords: Coordinates {
                        x: 3f32,
                        y: 2f32,
                        z: 5f32,
                        w: 0.1f32,
                    },
                },
            ]),
        };
        let result = receiver.at_time(6);
        assert_eq!(
            Receiver {
                keyframes: None,
                index: 0,
                coordinates: Some(Coordinates {
                    x: 30f32,
                    y: 36f32,
                    z: 9.999999f32,
                    w: 1f32
                })
            },
            result
        )
    }

    #[test]
    fn interpolate_emitter() {
        let emitter = Emitter {
            index: 0,
            coordinates: None,
            keyframes: Some(vec![
                CoordinateKeyframe {
                    time: 5,
                    coords: Coordinates {
                        x: 3f32,
                        y: 4f32,
                        z: 0f32,
                        w: 0.1f32,
                    },
                },
                CoordinateKeyframe {
                    time: 10,
                    coords: Coordinates {
                        x: 3f32,
                        y: 2f32,
                        z: 5f32,
                        w: 0.1f32,
                    },
                },
            ]),
        };
        let result = emitter.at_time(6);
        assert_eq!(
            Emitter {
                keyframes: None,
                index: 0,
                coordinates: Some(Coordinates {
                    x: 30f32,
                    y: 36f32,
                    z: 9.999999f32,
                    w: 1f32
                })
            },
            result
        )
    }

    #[test]
    fn interpolate_object_keyframes_before() {
        let keyframes = vec![
            ObjectKeyframe {
                time: 5,
                coords: [
                    Coordinates {
                        x: 1f32,
                        y: 2f32,
                        z: 3f32,
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
                coords: [
                    Coordinates {
                        x: 3f32,
                        y: 2f32,
                        z: 5f32,
                        w: 0.1f32,
                    },
                    Coordinates {
                        x: 4f32,
                        y: 5f32,
                        z: 6f32,
                        w: 0.5f32,
                    },
                ],
            },
        ];
        let time = 0;
        assert_eq!(
            vec![
                Coordinates {
                    x: 1f32,
                    y: 2f32,
                    z: 3f32,
                    w: 0.1f32,
                },
                Coordinates {
                    x: 0f32,
                    y: 1f32,
                    z: 8f32,
                    w: 0.5f32,
                },
            ],
            interpolate_object_keyframes(&keyframes, time)
        )
    }

    #[test]
    fn interpolate_object_keyframes_during() {
        let keyframes = vec![
            ObjectKeyframe {
                time: 5,
                coords: [
                    Coordinates {
                        x: 1f32,
                        y: 2f32,
                        z: 3f32,
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
                coords: [
                    Coordinates {
                        x: 3f32,
                        y: 2f32,
                        z: 5f32,
                        w: 0.1f32,
                    },
                    Coordinates {
                        x: 4f32,
                        y: 5f32,
                        z: 6f32,
                        w: 0.5f32,
                    },
                ],
            },
        ];
        let time = 7;
        assert_eq!(
            vec![
                Coordinates {
                    x: 18f32,
                    y: 20f32,
                    z: 38f32,
                    w: 1f32,
                },
                Coordinates {
                    x: 3.1999998f32,
                    y: 5.2f32,
                    z: 14.4f32,
                    w: 1f32,
                },
            ],
            interpolate_object_keyframes(&keyframes, time)
        )
    }

    #[test]
    fn interpolate_object_keyframes_after() {
        let keyframes = vec![
            ObjectKeyframe {
                time: 5,
                coords: [
                    Coordinates {
                        x: 1f32,
                        y: 2f32,
                        z: 3f32,
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
                coords: [
                    Coordinates {
                        x: 3f32,
                        y: 2f32,
                        z: 5f32,
                        w: 0.1f32,
                    },
                    Coordinates {
                        x: 4f32,
                        y: 5f32,
                        z: 6f32,
                        w: 0.5f32,
                    },
                ],
            },
        ];
        let time = 15;
        assert_eq!(
            vec![
                Coordinates {
                    x: 3f32,
                    y: 2f32,
                    z: 5f32,
                    w: 0.1f32,
                },
                Coordinates {
                    x: 4f32,
                    y: 5f32,
                    z: 6f32,
                    w: 0.5f32,
                },
            ],
            interpolate_object_keyframes(&keyframes, time)
        )
    }

    #[test]
    fn interpolate_coordinate_keyframes_before() {
        let keyframes = vec![
            CoordinateKeyframe {
                time: 5,
                coords: Coordinates {
                    x: 1f32,
                    y: 2f32,
                    z: 3f32,
                    w: 0.1f32,
                },
            },
            CoordinateKeyframe {
                time: 10,
                coords: Coordinates {
                    x: 3f32,
                    y: 2f32,
                    z: 5f32,
                    w: 0.1f32,
                },
            },
        ];
        let time = 0;
        assert_eq!(
            Coordinates {
                x: 1f32,
                y: 2f32,
                z: 3f32,
                w: 0.1f32
            },
            interpolate_coordinate_keyframes(&keyframes, time)
        )
    }

    #[test]
    fn interpolate_coordinate_keyframes_during() {
        let keyframes = vec![
            CoordinateKeyframe {
                time: 5,
                coords: Coordinates {
                    x: 3f32,
                    y: 4f32,
                    z: 0f32,
                    w: 0.1f32,
                },
            },
            CoordinateKeyframe {
                time: 10,
                coords: Coordinates {
                    x: 3f32,
                    y: 2f32,
                    z: 5f32,
                    w: 0.1f32,
                },
            },
        ];
        let time = 6;
        assert_eq!(
            Coordinates {
                x: 30f32,
                y: 36f32,
                z: 9.999999f32,
                w: 1f32
            },
            interpolate_coordinate_keyframes(&keyframes, time)
        )
    }

    #[test]
    fn interpolate_coordinate_keyframes_after() {
        let keyframes = vec![
            CoordinateKeyframe {
                time: 5,
                coords: Coordinates {
                    x: 3f32,
                    y: 4f32,
                    z: 0f32,
                    w: 0.1f32,
                },
            },
            CoordinateKeyframe {
                time: 10,
                coords: Coordinates {
                    x: 3f32,
                    y: 2f32,
                    z: 5f32,
                    w: 0.1f32,
                },
            },
        ];
        let time = 10;
        assert_eq!(
            Coordinates {
                x: 3f32,
                y: 2f32,
                z: 5f32,
                w: 0.1f32
            },
            interpolate_coordinate_keyframes(&keyframes, time)
        )
    }

    #[test]
    fn interpolate_coordinates_w_1() {
        let coords1 = Coordinates {
            x: 0.5f32,
            y: 3f32,
            z: 10f32,
            w: 1f32,
        };
        let coords2 = Coordinates {
            x: 10f32,
            y: 1.6f32,
            z: 20f32,
            w: 1f32,
        };
        let interp_position = 0.25f32;
        let expected = Coordinates {
            x: 7.625f32,
            y: 1.95f32,
            z: 17.5f32,
            w: 1f32,
        };
        assert_eq!(
            expected,
            interpolate_coordinates(&coords1, &coords2, interp_position)
        )
    }

    #[test]
    fn interpolate_coordinates_w_varied() {
        let coords1 = Coordinates {
            x: 0.05f32,
            y: 0.3f32,
            z: 1f32,
            w: 0.1f32,
        };
        let coords2 = Coordinates {
            x: 25f32,
            y: 4f32,
            z: 50f32,
            w: 2.5f32,
        };
        let interp_position = 0.25f32;
        let expected = Coordinates {
            x: 7.625f32,
            y: 1.95f32,
            z: 17.5f32,
            w: 1f32,
        };
        assert_eq!(
            expected,
            interpolate_coordinates(&coords1, &coords2, interp_position)
        )
    }

    #[test]
    fn interpolate_single_coordinate_w_1() {
        assert_eq!(
            2f32,
            interpolate_single_coordinate(1f32, 1f32, 3f32, 1f32, 0.5)
        )
    }

    #[test]
    fn interpolate_single_coordinate_varied_w() {
        assert_eq!(
            17f32,
            interpolate_single_coordinate(10f32, 0.5f32, 40f32, 4f32, 0.7)
        )
    }

    #[test]
    fn calculate_interp_position_0() {
        assert_eq!(1f32, calculate_interp_position(1000, 50000, 1000))
    }

    #[test]
    fn calculate_interp_position_inbetween() {
        assert_eq!(0.625f32, calculate_interp_position(10000, 50000, 25000))
    }

    #[test]
    fn calculate_interp_position_1() {
        assert_eq!(0f32, calculate_interp_position(1000, 50000, 50000))
    }
}
