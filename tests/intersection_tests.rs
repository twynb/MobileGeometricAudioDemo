use approx::assert_abs_diff_eq;
use demo::intersection::{intersect_ray_and_receiver, intersect_ray_and_surface};
use demo::ray::Ray;
use demo::scene::{CoordinateKeyframe, Receiver, Surface};
use nalgebra::{Unit, Vector3};

fn assert_intersection_equal(
    expected: Option<(u32, Vector3<f32>)>,
    result: Option<(u32, Vector3<f32>)>,
) {
    match expected {
        None => assert!(
            result.is_none(),
            "Result is Some where it should have been None. Result: {result:?}"
        ),
        Some((time, coords)) => {
            assert!(
                result.is_some(),
                "Result is None where it should have been Some. Expected: {expected:?}"
            );
            let result = result.unwrap();
            assert_eq!(
                time, result.0,
                "Time between Result and Expected doesn't match"
            );
            for idx in 0..3 {
                assert_abs_diff_eq!(coords[idx], result.1[idx], epsilon = 0.01)
            }
        }
    }
}

#[test]
fn test_intersect_static_receiver() {
    let receiver = Receiver::Interpolated(Vector3::new(10f32, 10f32, 1f32), 0.1f32, 0);

    let hitting_ray: Ray = Ray {
        direction: Unit::new_normalize(Vector3::new(5f32, 10f32, -1f32)),
        origin: Vector3::new(5f32, 0f32, 2f32),
        energy: 1f32,
        time: 0,
        velocity: 1f32,
    };

    assert_intersection_equal(
        Some((11, Vector3::new(9.95549, 9.910981, 1.0089018f32))),
        intersect_ray_and_receiver(&hitting_ray, &receiver, 0, 100),
    );

    assert_intersection_equal(
        None,
        intersect_ray_and_receiver(&hitting_ray, &receiver, 1, 10),
    );

    let narrowly_hitting_ray = Ray {
        direction: Unit::new_normalize(Vector3::new(0f32, 10f32, -1f32)),
        origin: Vector3::new(10.1f32, 0f32, 2f32),
        energy: 1f32,
        time: 0,
        velocity: 1f32,
    };

    assert_intersection_equal(
        Some((10, Vector3::new(10.1f32, 10f32, 1f32))),
        intersect_ray_and_receiver(&narrowly_hitting_ray, &receiver, 0, 100),
    );

    let narrowly_missing_ray = Ray {
        direction: Unit::new_normalize(Vector3::new(0.001f32, 10f32, -1f32)),
        origin: Vector3::new(10.1f32, 0f32, 2f32),
        energy: 1f32,
        time: 0,
        velocity: 1f32,
    };

    assert_intersection_equal(
        None,
        intersect_ray_and_receiver(&narrowly_missing_ray, &receiver, 0, 100),
    );

    let missing_ray: Ray = Ray {
        direction: Unit::new_normalize(Vector3::new(1f32, 1f32, 1f32)),
        origin: Vector3::new(15f32, 0f32, 2f32),
        energy: 1f32,
        time: 0,
        velocity: 1f32,
    };

    assert_intersection_equal(
        None,
        intersect_ray_and_receiver(&missing_ray, &receiver, 0, 100),
    )
}

#[test]
fn test_intersect_moving_receiver() {
    let receiver_moving_towards_emitter = Receiver::Keyframes(
        vec![
            CoordinateKeyframe {
                time: 0,
                coords: Vector3::new(-10f32, 0f32, 0f32),
            },
            CoordinateKeyframe {
                time: 20,
                coords: Vector3::new(0f32, 0f32, 0f32),
            },
        ],
        0.1f32,
    );

    let hitting_ray: Ray = Ray {
        direction: Unit::new_normalize(Vector3::new(-1f32, 0f32, 0f32)),
        origin: Vector3::new(5f32, 0f32, 0f32),
        energy: 1f32,
        time: 0,
        velocity: 1f32,
    };

    assert_intersection_equal(
        Some((10, Vector3::new(-4.8, 0.0, 0.0))),
        intersect_ray_and_receiver(&hitting_ray, &receiver_moving_towards_emitter, 0, 100),
    );

    let receiver = Receiver::Keyframes(
        vec![
            CoordinateKeyframe {
                time: 0,
                coords: Vector3::new(0f32, 20f32, 1f32),
            },
            CoordinateKeyframe {
                time: 20,
                coords: Vector3::new(20f32, 0f32, 1f32),
            },
        ],
        0.1f32,
    );

    let narrowly_hitting_ray = Ray {
        direction: Unit::new_normalize(Vector3::new(0f32, 10f32, 0f32)),
        origin: Vector3::new(10.1f32, 0f32, 1f32),
        energy: 1f32,
        time: 0,
        velocity: 1f32,
    };

    assert_intersection_equal(
        Some((10, Vector3::new(10.1f32, 9.8f32, 1f32))),
        intersect_ray_and_receiver(&narrowly_hitting_ray, &receiver, 0, 100),
    );

    let narrowly_missing_ray = Ray {
        direction: Unit::new_normalize(Vector3::new(0.001f32, 10f32, -0.5f32)),
        origin: Vector3::new(10.1f32, 0f32, 2f32),
        energy: 1f32,
        time: 0,
        velocity: 1f32,
    };

    assert_intersection_equal(
        None,
        intersect_ray_and_receiver(&narrowly_missing_ray, &receiver, 0, 100),
    );

    let missing_ray: Ray = Ray {
        direction: Unit::new_normalize(Vector3::new(1f32, 1f32, 1f32)),
        origin: Vector3::new(15f32, 0f32, 2f32),
        energy: 1f32,
        time: 0,
        velocity: 1f32,
    };

    assert_intersection_equal(
        None,
        intersect_ray_and_receiver(&missing_ray, &receiver, 0, 100),
    );

    let too_late_ray: Ray = Ray {
        direction: Unit::new_normalize(Vector3::new(0f32, 10f32, 0f32)),
        origin: Vector3::new(10.1f32, 0f32, 1f32),
        energy: 1f32,
        time: 2,
        velocity: 0.5f32,
    };

    assert_intersection_equal(
        None,
        intersect_ray_and_receiver(&too_late_ray, &receiver, 2, 100),
    );

    let late_hitting_ray: Ray = Ray {
        direction: Unit::new_normalize(Vector3::new(1f32, 1f32, 0f32)),
        origin: Vector3::new(10f32, -10f32, 1f32),
        energy: 1f32,
        time: 20,
        velocity: 1f32,
    };

    assert_intersection_equal(
        Some((34, Vector3::new(19.93f32, -0.07f32, 1f32))),
        intersect_ray_and_receiver(&late_hitting_ray, &receiver, 0, 100),
    );
}

#[test]
fn test_intersect_static_surface() {
    let surface = Surface::Interpolated(
        [
            Vector3::new(10f32, 3f32, 0f32),
            Vector3::new(0f32, 3f32, 0f32),
            Vector3::new(0f32, 3f32, 10f32),
        ],
        0,
    );

    let hitting_ray: Ray = Ray {
        direction: Unit::new_normalize(Vector3::new(0f32, 10f32, 0f32)),
        origin: Vector3::new(5f32, -4f32, 2f32),
        energy: 1f32,
        time: 0,
        velocity: 1f32,
    };

    assert_intersection_equal(
        Some((7, Vector3::new(5f32, 3f32, 2f32))),
        intersect_ray_and_surface(&hitting_ray, &surface, 0, 100),
    );

    assert_intersection_equal(
        None,
        intersect_ray_and_surface(&hitting_ray, &surface, 1, 5),
    );

    let narrowly_hitting_ray = Ray {
        direction: Unit::new_normalize(Vector3::new(0f32, 1f32, 0f32)),
        origin: Vector3::new(0f32, 0f32, 0f32),
        energy: 1f32,
        time: 0,
        velocity: 1f32,
    };

    assert_intersection_equal(
        Some((3, Vector3::new(0f32, 3f32, 0f32))),
        intersect_ray_and_surface(&narrowly_hitting_ray, &surface, 0, 100),
    );

    let narrowly_missing_ray = Ray {
        direction: Unit::new_normalize(Vector3::new(0f32, 1f32, 0f32)),
        origin: Vector3::new(-0.01f32, 0f32, 0f32),
        energy: 1f32,
        time: 0,
        velocity: 1f32,
    };

    assert_intersection_equal(
        None,
        intersect_ray_and_surface(&narrowly_missing_ray, &surface, 0, 100),
    );

    let missing_ray: Ray = Ray {
        direction: Unit::new_normalize(Vector3::new(1f32, 0f32, 1f32)),
        origin: Vector3::new(15f32, 0f32, 2f32),
        energy: 1f32,
        time: 0,
        velocity: 1f32,
    };

    assert_intersection_equal(
        None,
        intersect_ray_and_surface(&missing_ray, &surface, 0, 100),
    )
}
