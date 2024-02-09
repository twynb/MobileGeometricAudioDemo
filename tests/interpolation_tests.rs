use demo::interpolation::Interpolation;
use demo::scene::{CoordinateKeyframe, Emitter, Receiver, Surface, SurfaceKeyframe};
use nalgebra::Vector3;

#[test]
fn interpolate_surface() {
    let object = Surface::Keyframes(vec![
        SurfaceKeyframe {
            time: 5,
            coords: [
                Vector3::new(10f32, 20f32, 30f32),
                Vector3::new(0f32, 2f32, 16f32),
            ],
        },
        SurfaceKeyframe {
            time: 10,
            coords: [
                Vector3::new(30f32, 20f32, 50f32),
                Vector3::new(8f32, 10f32, 12f32),
            ],
        },
    ]);
    let result = object.at_time(7);
    assert_eq!(
        Surface::Interpolated(
            [
                Vector3::new(18f32, 20f32, 38f32),
                Vector3::new(3.1999998f32, 5.2f32, 14.4f32),
            ],
            7
        ),
        result
    );
}

#[test]
fn interpolate_receiver() {
    let receiver = Receiver::Keyframes(
        vec![
            CoordinateKeyframe {
                time: 5,
                coords: Vector3::new(30f32, 40f32, 0f32),
            },
            CoordinateKeyframe {
                time: 10,
                coords: Vector3::new(30f32, 20f32, 50f32),
            },
        ],
        0.1f32,
    );
    let result = receiver.at_time(6);
    assert_eq!(
        Receiver::Interpolated(Vector3::new(30f32, 36f32, 9.999999f32), 0.1f32, 6),
        result
    );
}

#[test]
fn interpolate_emitter() {
    let emitter = Emitter::Keyframes(vec![
        CoordinateKeyframe {
            time: 5,
            coords: Vector3::new(30f32, 40f32, 0f32),
        },
        CoordinateKeyframe {
            time: 10,
            coords: Vector3::new(30f32, 20f32, 50f32),
        },
    ]);
    let result = emitter.at_time(6);
    assert_eq!(
        Emitter::Interpolated(Vector3::new(30f32, 36f32, 9.999999f32,), 6),
        result
    );
}
