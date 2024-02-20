use approx::abs_diff_eq;
use demo::bounce::EmissionType;
use demo::interpolation::Interpolation;
use demo::materials::MATERIAL_CONCRETE_WALL;
use demo::scene::{CoordinateKeyframe, Emitter, Receiver, Surface, SurfaceKeyframe};
use nalgebra::Vector3;

fn vector_abs_diff_eq(a: Vector3<f64>, b: Vector3<f64>) -> bool {
    for i in 0..3 {
        if !(abs_diff_eq!(a[i], b[i], epsilon = 0.000001)) {
            return false;
        }
    }
    true
}

fn assert_vector_abs_diff_eq(a: Vector3<f64>, b: Vector3<f64>) {
    assert!(
        vector_abs_diff_eq(a, b),
        "assertion `left == right` failed. left: {a:?}, right: {b:?}"
    );
}

#[test]
fn interpolate_surface() {
    let object = Surface::Keyframes(
        vec![
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
        ],
        MATERIAL_CONCRETE_WALL,
    );
    let result = object.at_time(7);
    let Surface::Interpolated(result_coords, time, material) = result else {
        panic!("Surface wasn't interpolated!")
    };
    let expected_coords = [
        Vector3::new(18f64, 20f64, 38f64),
        Vector3::new(3.1999998f64, 5.2f64, 14.4f64),
    ];
    for idx in 0..expected_coords.len() {
        assert_vector_abs_diff_eq(expected_coords[idx], result_coords[idx]);
    }
    assert_eq!(7, time);
    assert_eq!(MATERIAL_CONCRETE_WALL, material);
}

#[test]
fn interpolate_receiver() {
    let receiver = Receiver::Keyframes(
        vec![
            CoordinateKeyframe {
                time: 5,
                coords: Vector3::new(30f64, 40f64, 0f64),
            },
            CoordinateKeyframe {
                time: 10,
                coords: Vector3::new(30f64, 20f64, 50f64),
            },
        ],
        0.1f64,
    );
    let result = receiver.at_time(6);
    let Receiver::Interpolated(result_coords, radius, time) = result else {
        panic!("Receiver wasn't interpolated!")
    };
    assert_vector_abs_diff_eq(Vector3::new(30f64, 36f64, 10f64), result_coords);
    assert_eq!(0.1f64, radius);
    assert_eq!(6, time);
}

#[test]
fn interpolate_emitter() {
    let emitter = Emitter::Keyframes(
        vec![
            CoordinateKeyframe {
                time: 5,
                coords: Vector3::new(30f64, 40f64, 0f64),
            },
            CoordinateKeyframe {
                time: 10,
                coords: Vector3::new(30f64, 20f64, 50f64),
            },
        ],
        EmissionType::Random,
    );
    let result = emitter.at_time(6);
    let Emitter::Interpolated(result_coords, time, emission_type) = result else {
        panic!("Emitter wasn't interpolated!")
    };
    assert_vector_abs_diff_eq(Vector3::new(30f64, 36f64, 10f64), result_coords);
    assert_eq!(6, time);
    assert_eq!(EmissionType::Random, emission_type);
}
