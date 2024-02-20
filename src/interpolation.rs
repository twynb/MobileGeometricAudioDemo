use nalgebra::Vector3;
use num::{Num, NumCast};

use crate::scene::{CoordinateKeyframe, Emitter, Receiver, Surface, SurfaceKeyframe};

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
    coords1: &[Vector3<f64>],
    coords2: &[Vector3<f64>],
    interp_position: f64,
) -> [Vector3<f64>; N] {
    let mut result: [Vector3<f64>; N] = [Vector3::new(0f64, 0f64, 0f64); N];

    for (idx, coord) in coords1.iter().enumerate() {
        result[idx] = interpolate_coordinates(coord, &coords2[idx], interp_position);
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
    coords1: &Vector3<f64>,
    coords2: &Vector3<f64>,
    interp_position: f64,
) -> Vector3<f64> {
    Vector3::new(
        interpolate_single_coordinate(coords1.x, coords2.x, interp_position),
        interpolate_single_coordinate(coords1.y, coords2.y, interp_position),
        interpolate_single_coordinate(coords1.z, coords2.z, interp_position),
    )
}

/// Get the interpolated value, taking the w value into account.
///
/// # Arguments
/// * `coord1`: The first coordinate.
/// * `w1`: `w` for the first coordinate.
/// * `coord2`: The second coordinate.
/// * `w2`: `w` for the second coordinate.
/// * `interp_position`: The interpolation position. Should be between 0 (only use first coordinate) and 1 (only second coordinate).
fn interpolate_single_coordinate(coord1: f64, coord2: f64, interp_position: f64) -> f64 {
    coord1.mul_add(interp_position, coord2 * (1f64 - interp_position))
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
///
/// # Panics
///
/// * if `interpolate_two_coordinate_keyframes` fails. This shouldn't be able to happen and can be ignored.
pub fn interpolate_coordinate_keyframes(
    keyframes: &Vec<CoordinateKeyframe>,
    time: u32,
) -> Vector3<f64> {
    // return out early if we're after the last keyframe anyway
    if time >= keyframes[keyframes.len() - 1].time {
        return keyframes[keyframes.len() - 1].coords;
    }

    for pair in keyframes.windows(2) {
        let result = interpolate_two_coordinate_keyframes(&pair[0], &pair[1], time);
        if let Some(result) = result {
            return result;
        }
    }

    // unable to happen
    panic!("Error in interpolation - this should not happen.");
}

/// Calculate the interpolated coordinate between the keyframes at the given time.
/// If the time is before or equal to the first keyframe, return its coordinates.
/// If the time is equal to the second keyframe's time, return its coordinates.
/// If the time is after the second keyframe, return None.
/// Otherweise, interpolate.
///
/// # Arguments
/// * `first`: The first keyframe to interpolate.
/// * `second`: The second keyframe to interpolate.
/// * `time`: The time.
///
/// # Panics
///
/// * If u32 can't be cast to T or T can't be cast to f64.
pub fn interpolate_two_coordinate_keyframes<T: Num + NumCast + PartialOrd + Copy>(
    first: &CoordinateKeyframe,
    second: &CoordinateKeyframe,
    time: T,
) -> Option<Vector3<f64>> {
    let first_time: T = num::cast(first.time).unwrap();
    let second_time: T = num::cast(second.time).unwrap();
    if time <= first_time {
        return Some(first.coords);
    }
    if time == second_time {
        return Some(second.coords);
    }
    if time >= first_time && time < second_time {
        let interp_position = calculate_interp_position(first_time, second_time, time);
        return Some(interpolate_coordinates(
            &first.coords,
            &second.coords,
            interp_position,
        ));
    }
    None
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
fn interpolate_surface_keyframes<const N: usize>(
    keyframes: &Vec<SurfaceKeyframe<N>>,
    time: u32,
) -> [Vector3<f64>; N] {
    // return out early if we're after the last keyframe, otherwise we'd need to iterate over all the keyframes first
    if time >= keyframes[keyframes.len() - 1].time {
        return keyframes[keyframes.len() - 1].coords;
    }

    for pair in keyframes.windows(2) {
        let result = interpolate_two_surface_keyframes(&pair[0], &pair[1], time);
        if let Some(result) = result {
            return result;
        }
    }

    // unable to happen
    panic!("Error in interpolation - this should not happen.");
}

/// Calculate the interpolated coordinate between the keyframes at the given time.
/// If the time is before or equal to the first keyframe, return its coordinates.
/// If the time is equal to the second keyframe's time, return its coordinates.
/// If the time is after the second keyframe, return None.
/// Otherweise, interpolate.
///
/// # Arguments
/// * `first`: The first keyframe to interpolate.
/// * `second`: The second keyframe to interpolate.
/// * `time`: The time.
///
/// # Panics
///
/// * If u32 can't be cast to T or T can't be cast to f64.
pub fn interpolate_two_surface_keyframes<const N: usize, T: Num + NumCast + PartialOrd + Copy>(
    first: &SurfaceKeyframe<N>,
    second: &SurfaceKeyframe<N>,
    time: T,
) -> Option<[Vector3<f64>; N]> {
    let first_time: T = num::cast(first.time).unwrap();
    let second_time: T = num::cast(second.time).unwrap();
    if time <= first_time {
        return Some(first.coords);
    }
    if time == second_time {
        return Some(second.coords);
    }
    if time >= first_time && time < second_time {
        let interp_position = calculate_interp_position(first_time, second_time, time);
        return Some(interpolate_coordinate_array(
            &first.coords,
            &second.coords,
            interp_position,
        ));
    }
    None
}

/// Calculate the interpolation position, i.e. how much of the keyframe at `first_time`
/// is still left in the coordinates at `time`.
/// This assumes that `first_time` <= `time` <= `second_time`
///
/// # Arguments
/// * `first_time`: Time of the first keyframe.
/// * `second_time`: Time of the second keyframe.
/// * `time`: The current time.
fn calculate_interp_position<T: Num + NumCast + Copy>(
    first_time: T,
    second_time: T,
    time: T,
) -> f64 {
    num::cast::<T, f64>(second_time - time).unwrap()
        / num::cast::<T, f64>(second_time - first_time).unwrap()
}

impl Interpolation for Emitter {
    fn at_time(&self, time: u32) -> Self {
        match self {
            Self::Interpolated(_keyframes, _time, _type) => self.clone(),
            Self::Keyframes(keyframes, emission_type) => Self::Interpolated(
                interpolate_coordinate_keyframes(keyframes, time),
                time,
                *emission_type,
            ),
        }
    }
}

impl Interpolation for Receiver {
    fn at_time(&self, time: u32) -> Self {
        match self {
            Self::Interpolated(_keyframes, _radius, _time) => self.clone(),
            Self::Keyframes(keyframes, radius) => Self::Interpolated(
                interpolate_coordinate_keyframes(keyframes, time),
                *radius,
                time,
            ),
        }
    }
}

impl<const N: usize> Interpolation for Surface<N> {
    fn at_time(&self, time: u32) -> Self {
        match self {
            Self::Interpolated(_keyframes, _time, _material) => self.clone(),
            Self::Keyframes(keyframes, material) => Self::Interpolated(
                interpolate_surface_keyframes(keyframes, time),
                time,
                *material,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::Vector3;

    use crate::{
        scene::{CoordinateKeyframe, SurfaceKeyframe},
        test_utils::{self, assert_vector_abs_diff_eq},
    };

    // TODO tests: at_time() for surface

    use super::{
        calculate_interp_position, interpolate_coordinate_keyframes, interpolate_coordinates,
        interpolate_single_coordinate, interpolate_surface_keyframes,
    };

    #[test]
    fn interpolate_object_keyframes_before() {
        let keyframes = vec![
            SurfaceKeyframe {
                time: 5,
                coords: [
                    Vector3::new(10f64, 20f64, 30f64),
                    Vector3::new(0f64, 2f64, 16f64),
                ],
            },
            SurfaceKeyframe {
                time: 10,
                coords: [
                    Vector3::new(30f64, 20f64, 50f64),
                    Vector3::new(8f64, 10f64, 12f64),
                ],
            },
        ];
        let time = 0;
        assert_eq!(
            vec![
                Vector3::new(10f64, 20f64, 30f64),
                Vector3::new(0f64, 2f64, 16f64),
            ],
            interpolate_surface_keyframes(&keyframes, time)
        )
    }

    #[test]
    fn interpolate_object_keyframes_during() {
        let keyframes = vec![
            SurfaceKeyframe {
                time: 5,
                coords: [
                    Vector3::new(10f64, 20f64, 30f64),
                    Vector3::new(0f64, 2f64, 16f64),
                ],
            },
            SurfaceKeyframe {
                time: 10,
                coords: [
                    Vector3::new(30f64, 20f64, 50f64),
                    Vector3::new(8f64, 10f64, 12f64),
                ],
            },
        ];
        let time = 7;
        let expected = vec![
                Vector3::new(18f64, 20f64, 38f64),
                Vector3::new(3.1999998f64, 5.2f64, 14.4f64),
            ];
        let result = interpolate_surface_keyframes(&keyframes, time);
        assert_eq!(expected.len(), result.len());
        for idx in 0..expected.len() {
            assert_vector_abs_diff_eq(expected[idx], result[idx])
        }
    }

    #[test]
    fn interpolate_object_keyframes_after() {
        let keyframes = vec![
            SurfaceKeyframe {
                time: 5,
                coords: [
                    Vector3::new(10f64, 20f64, 30f64),
                    Vector3::new(0f64, 2f64, 16f64),
                ],
            },
            SurfaceKeyframe {
                time: 10,
                coords: [
                    Vector3::new(30f64, 20f64, 50f64),
                    Vector3::new(8f64, 10f64, 12f64),
                ],
            },
        ];
        let time = 15;
        assert_eq!(
            vec![
                Vector3::new(30f64, 20f64, 50f64),
                Vector3::new(8f64, 10f64, 12f64),
            ],
            interpolate_surface_keyframes(&keyframes, time)
        )
    }

    #[test]
    fn interpolate_coordinate_keyframes_before() {
        let keyframes = vec![
            CoordinateKeyframe {
                time: 5,
                coords: Vector3::new(10f64, 20f64, 30f64),
            },
            CoordinateKeyframe {
                time: 10,
                coords: Vector3::new(30f64, 20f64, 50f64),
            },
        ];
        let time = 0;
        assert_eq!(
            Vector3::new(10f64, 20f64, 30f64),
            interpolate_coordinate_keyframes(&keyframes, time)
        )
    }

    #[test]
    fn interpolate_coordinate_keyframes_during() {
        let keyframes = vec![
            CoordinateKeyframe {
                time: 5,
                coords: Vector3::new(30f64, 40f64, 0f64),
            },
            CoordinateKeyframe {
                time: 10,
                coords: Vector3::new(30f64, 20f64, 50f64),
            },
        ];
        let time = 6;
        test_utils::assert_vector_abs_diff_eq(
            Vector3::new(30f64, 36f64, 10f64),
            interpolate_coordinate_keyframes(&keyframes, time),
        )
    }

    #[test]
    fn interpolate_coordinate_keyframes_after() {
        let keyframes = vec![
            CoordinateKeyframe {
                time: 5,
                coords: Vector3::new(30f64, 40f64, 0f64),
            },
            CoordinateKeyframe {
                time: 10,
                coords: Vector3::new(30f64, 20f64, 50f64),
            },
        ];
        let time = 10;
        assert_eq!(
            Vector3::new(30f64, 20f64, 50f64),
            interpolate_coordinate_keyframes(&keyframes, time)
        )
    }

    #[test]
    fn interpolate_coordinates_w_1() {
        let coords1 = Vector3::new(0.5f64, 3f64, 10f64);
        let coords2 = Vector3::new(10f64, 1.6f64, 20f64);
        let interp_position = 0.25f64;
        let expected = Vector3::new(7.625f64, 1.95f64, 17.5f64);
        assert_vector_abs_diff_eq(
            expected,
            interpolate_coordinates(&coords1, &coords2, interp_position)
        )
    }

    #[test]
    fn interpolate_coordinates_w_varied() {
        let coords1 = Vector3::new(0.5f64, 3f64, 10f64);
        let coords2 = Vector3::new(10f64, 1f64, 20f64);
        let interp_position = 0.25f64;
        let expected = Vector3::new(7.625f64, 1.5f64, 17.5f64);
        assert_eq!(
            expected,
            interpolate_coordinates(&coords1, &coords2, interp_position)
        )
    }

    #[test]
    fn interpolate_single_coordinate_w_1() {
        assert_eq!(2f64, interpolate_single_coordinate(1f64, 3f64, 0.5))
    }

    #[test]
    fn interpolate_single_coordinate_varied_w() {
        assert_eq!(17f64, interpolate_single_coordinate(20f64, 10f64, 0.7))
    }

    #[test]
    fn calculate_interp_position_0() {
        assert_eq!(1f64, calculate_interp_position(1000, 50000, 1000))
    }

    #[test]
    fn calculate_interp_position_inbetween() {
        assert_eq!(0.625f64, calculate_interp_position(10000, 50000, 25000))
    }

    #[test]
    fn calculate_interp_position_1() {
        assert_eq!(0f64, calculate_interp_position(1000, 50000, 50000))
    }
}
