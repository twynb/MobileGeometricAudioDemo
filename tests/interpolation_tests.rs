use demo::scene::{CoordinateKeyframe, Coordinates, Emitter, Receiver, Surface, SurfaceKeyframe};
use demo::interpolation::Interpolation;

#[test]
fn interpolate_object() {
    let object = Surface::Keyframes(vec![
        SurfaceKeyframe {
            time: 5,
            coords: [
                Coordinates {
                    x: 10f32,
                    y: 20f32,
                    z: 30f32,
                },
                Coordinates {
                    x: 0f32,
                    y: 2f32,
                    z: 16f32,
                },
            ],
        },
        SurfaceKeyframe {
            time: 10,
            coords: [
                Coordinates {
                    x: 30f32,
                    y: 20f32,
                    z: 50f32,
                },
                Coordinates {
                    x: 8f32,
                    y: 10f32,
                    z: 12f32,
                },
            ],
        },
    ]);
    let result = object.at_time(7);
    assert_eq!(
        Surface::Interpolated(
            [
                Coordinates {
                    x: 18f32,
                    y: 20f32,
                    z: 38f32,
                },
                Coordinates {
                    x: 3.1999998f32,
                    y: 5.2f32,
                    z: 14.4f32,
                },
            ],
            7
        ),
        result
    )
}

#[test]
fn interpolate_receiver() {
    let receiver = Receiver::Keyframes(
        vec![
            CoordinateKeyframe {
                time: 5,
                coords: Coordinates {
                    x: 30f32,
                    y: 40f32,
                    z: 0f32,
                },
            },
            CoordinateKeyframe {
                time: 10,
                coords: Coordinates {
                    x: 30f32,
                    y: 20f32,
                    z: 50f32,
                },
            },
        ],
        0.1f32,
    );
    let result = receiver.at_time(6);
    assert_eq!(
        Receiver::Interpolated(
            Coordinates {
                x: 30f32,
                y: 36f32,
                z: 9.999999f32,
            },
            0.1f32,
            6
        ),
        result
    )
}

#[test]
fn interpolate_emitter() {
    let emitter = Emitter::Keyframes(vec![
        CoordinateKeyframe {
            time: 5,
            coords: Coordinates {
                x: 30f32,
                y: 40f32,
                z: 0f32,
            },
        },
        CoordinateKeyframe {
            time: 10,
            coords: Coordinates {
                x: 30f32,
                y: 20f32,
                z: 50f32,
            },
        },
    ]);
    let result = emitter.at_time(6);
    assert_eq!(
        Emitter::Interpolated(
            Coordinates {
                x: 30f32,
                y: 36f32,
                z: 9.999999f32,
            },
            6
        ),
        result
    )
}
