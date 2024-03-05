use approx::assert_abs_diff_eq;
use demo::intersection::{intersect_ray_and_receiver, intersect_ray_and_surface};
use demo::materials::MATERIAL_CONCRETE_WALL;
use demo::ray::{Ray, DEFAULT_PROPAGATION_SPEED};
use demo::scene::{CoordinateKeyframe, Receiver, Surface, SurfaceData, SurfaceKeyframe};
use demo::DEFAULT_SAMPLE_RATE;
use nalgebra::{Unit, Vector3};

fn assert_intersection_equals(
    expected: Option<(u32, Vector3<f64>)>,
    result: Option<(u32, Vector3<f64>)>,
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

fn static_receiver() -> Receiver {
    Receiver::Interpolated(Vector3::new(10f64, 10f64, 1f64), 0.1f64, 0)
}

fn moving_receiver() -> Receiver {
    Receiver::Keyframes(
        vec![
            CoordinateKeyframe {
                time: 0,
                coords: Vector3::new(0f64, 20f64, 1f64),
            },
            CoordinateKeyframe {
                time: 20,
                coords: Vector3::new(20f64, 0f64, 1f64),
            },
        ],
        0.1f64,
    )
}

fn static_surface() -> Surface<3> {
    Surface::Interpolated(
        [
            Vector3::new(10f64, 3f64, 0f64),
            Vector3::new(0f64, 3f64, 0f64),
            Vector3::new(0f64, 3f64, 10f64),
        ],
        0,
        SurfaceData::new(MATERIAL_CONCRETE_WALL),
    )
}

fn moving_surface() -> Surface<3> {
    Surface::Keyframes(
        vec![
            SurfaceKeyframe {
                time: 0,
                coords: [
                    Vector3::new(0f64, 3f64, 0f64),
                    Vector3::new(-10f64, 3f64, 0f64),
                    Vector3::new(-10f64, 3f64, 10f64),
                ],
            },
            SurfaceKeyframe {
                time: 10,
                coords: [
                    Vector3::new(10f64, 3f64, 0f64),
                    Vector3::new(0f64, 3f64, 0f64),
                    Vector3::new(0f64, 3f64, 10f64),
                ],
            },
            SurfaceKeyframe {
                time: 20,
                coords: [
                    Vector3::new(10f64, 5f64, 0f64),
                    Vector3::new(0f64, 5f64, 0f64),
                    Vector3::new(0f64, 5f64, 10f64),
                ],
            },
        ],
        SurfaceData::new(MATERIAL_CONCRETE_WALL),
    )
}

#[test]
fn clearly_hit_static_receiver() {
    let receiver = static_receiver();

    let hitting_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(5f64, 10f64, -1f64)),
        Vector3::new(5f64, 0f64, 2f64),
        1f64,
        800,
        1f64,
    );

    assert_intersection_equals(
        Some((811, Vector3::new(9.95549, 9.910981, 1.0089018f64))),
        intersect_ray_and_receiver(&hitting_ray, &receiver, 0, 900, Some(400)),
    );
}

#[test]
fn clearly_hit_static_receiver_velocity_not_1() {
    let receiver = static_receiver();

    let hitting_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(5f64, 10f64, -1f64)),
        Vector3::new(5f64, 0f64, 2f64),
        1f64,
        905,
        0.5f64,
    );

    assert_intersection_equals(
        Some((927, Vector3::new(9.95549, 9.910981, 1.0089018f64))),
        intersect_ray_and_receiver(&hitting_ray, &receiver, 0, 1000, Some(400)),
    );
}

#[test]
fn clearly_hit_distant_static_receiver() {
    let receiver = Receiver::Interpolated(Vector3::new(1373.2, 0f64, 0f64), 0.1, 0);

    let hitting_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(1f64, 0f64, 0f64)),
        Vector3::new(0f64, 0f64, 0f64),
        1f64,
        500,
        DEFAULT_PROPAGATION_SPEED / DEFAULT_SAMPLE_RATE,
    );

    assert_intersection_equals(
        Some((
            500 + (1373.1 * DEFAULT_SAMPLE_RATE / DEFAULT_PROPAGATION_SPEED).round() as u32,
            Vector3::new(1373.1, 0f64, 0f64),
        )),
        intersect_ray_and_receiver(&hitting_ray, &receiver, 0, 300000, Some(400)),
    );
}

#[test]
fn miss_static_receiver_because_time() {
    let receiver = static_receiver();

    let hitting_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(5f64, 10f64, -1f64)),
        Vector3::new(5f64, 0f64, 2f64),
        1f64,
        600,
        1f64,
    );

    assert_intersection_equals(
        None,
        intersect_ray_and_receiver(&hitting_ray, &receiver, 1, 610, Some(400)),
    );
}

#[test]
fn narrowly_hit_static_receiver() {
    let receiver = static_receiver();

    let narrowly_hitting_ray = Ray::new(
        Unit::new_normalize(Vector3::new(0f64, 10f64, -1f64)),
        Vector3::new(10.1f64, 0f64, 2f64),
        1f64,
        703,
        1f64,
    );

    assert_intersection_equals(
        Some((713, Vector3::new(10.1f64, 10f64, 1f64))),
        intersect_ray_and_receiver(&narrowly_hitting_ray, &receiver, 700, 800, Some(300)),
    );
}

#[test]
fn narrowly_miss_static_receiver() {
    let receiver = static_receiver();

    let narrowly_missing_ray = Ray::new(
        Unit::new_normalize(Vector3::new(0.001f64, 10f64, -1f64)),
        Vector3::new(10.1f64, 0f64, 2f64),
        1f64,
        800,
        1f64,
    );

    assert_intersection_equals(
        None,
        intersect_ray_and_receiver(&narrowly_missing_ray, &receiver, 800, 1000, Some(500)),
    );
}

#[test]
fn clearly_miss_static_receiver() {
    let receiver = static_receiver();

    let missing_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(1f64, 1f64, 1f64)),
        Vector3::new(15f64, 0f64, 2f64),
        1f64,
        700,
        1f64,
    );

    assert_intersection_equals(
        None,
        intersect_ray_and_receiver(&missing_ray, &receiver, 700, 900, Some(400)),
    )
}

#[test]
fn hit_receiver_moving_towards_ray() {
    let receiver_moving_towards_ray = Receiver::Keyframes(
        vec![
            CoordinateKeyframe {
                time: 0,
                coords: Vector3::new(-10f64, 0f64, 0f64),
            },
            CoordinateKeyframe {
                time: 20,
                coords: Vector3::new(0f64, 0f64, 0f64),
            },
        ],
        0.1f64,
    );

    let hitting_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(-1f64, 0f64, 0f64)),
        Vector3::new(5f64, 0f64, 0f64),
        1f64,
        60,
        1f64,
    );

    assert_intersection_equals(
        Some((70, Vector3::new(-4.93, 0.0, 0.0))),
        intersect_ray_and_receiver(&hitting_ray, &receiver_moving_towards_ray, 0, 100, Some(20)),
    );
}

#[test]
fn hit_receiver_moving_towards_ray_after_later_start() {
    let loop_dur = 44100 * 9;
    let receiver_moving_towards_ray = Receiver::Keyframes(
        vec![
            CoordinateKeyframe {
                time: 0,
                coords: Vector3::new(343.3f64, 0f64, 0f64),
            },
            CoordinateKeyframe {
                time: loop_dur,
                coords: Vector3::new(0f64, 0f64, 0f64),
            },
        ],
        0.1f64,
    );

    let hitting_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(1f64, 0f64, 0f64)),
        Vector3::new(0f64, 0f64, 0f64),
        1f64,
        1 + loop_dur,
        DEFAULT_PROPAGATION_SPEED / DEFAULT_SAMPLE_RATE,
    );

    assert_intersection_equals(
        Some((39690 + loop_dur, Vector3::new(308.87, 0.0, 0.0))),
        intersect_ray_and_receiver(&hitting_ray, &receiver_moving_towards_ray, loop_dur, 10000000, Some(loop_dur)),
    );
}

#[test]
fn narrowly_hit_moving_receiver() {
    let receiver = moving_receiver();

    let narrowly_hitting_ray = Ray::new(
        Unit::new_normalize(Vector3::new(0f64, 10f64, 0f64)),
        Vector3::new(10.1f64, 0f64, 1f64),
        1f64,
        100,
        1f64,
    );

    assert_intersection_equals(
        Some((110, Vector3::new(10.1f64, 10f64, 1f64))),
        intersect_ray_and_receiver(&narrowly_hitting_ray, &receiver, 0, 200, Some(20)),
    );
}

#[test]
fn narrowly_miss_moving_receiver() {
    let receiver = moving_receiver();

    let narrowly_missing_ray = Ray::new(
        Unit::new_normalize(Vector3::new(0.001f64, 10f64, -0.5f64)),
        Vector3::new(10.1f64, 0f64, 2f64),
        1f64,
        40,
        1f64,
    );

    assert_intersection_equals(
        None,
        intersect_ray_and_receiver(&narrowly_missing_ray, &receiver, 0, 100, Some(20)),
    );
}

#[test]
fn clearly_miss_moving_receiver() {
    let receiver = moving_receiver();

    let missing_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(1f64, 1f64, 1f64)),
        Vector3::new(15f64, 0f64, 2f64),
        1f64,
        60,
        1f64,
    );

    assert_intersection_equals(
        None,
        intersect_ray_and_receiver(&missing_ray, &receiver, 0, 100, Some(20)),
    );
}

#[test]
fn miss_moving_receiver_because_timing() {
    let receiver = moving_receiver();

    let too_late_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(0f64, 10f64, 0f64)),
        Vector3::new(10.1f64, 0f64, 1f64),
        1f64,
        42,
        0.5f64,
    );

    assert_intersection_equals(
        None,
        intersect_ray_and_receiver(&too_late_ray, &receiver, 2, 100, Some(20)),
    );
}

#[test]
fn hit_moving_receiver_after_movement_finished() {
    let receiver = moving_receiver();
    let late_hitting_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(1f64, 1f64, 0f64)),
        Vector3::new(10f64, -10f64, 1f64),
        1f64,
        60,
        1f64,
    );

    assert_intersection_equals(
        Some((74, Vector3::new(19.93f64, -0.07f64, 1f64))),
        intersect_ray_and_receiver(&late_hitting_ray, &receiver, 0, 100, Some(40)),
    );
}

#[test]
fn clearly_hit_static_surface() {
    let surface = static_surface();

    let hitting_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(0f64, 10f64, 0f64)),
        Vector3::new(5f64, -4f64, 2f64),
        1f64,
        200,
        1f64,
    );

    assert_intersection_equals(
        Some((207, Vector3::new(5f64, 3f64, 2f64))),
        intersect_ray_and_surface(&hitting_ray, &surface, 0, 300, Some(100), 0),
    );
}

#[test]
fn miss_static_surface_because_time() {
    let surface = static_surface();

    let hitting_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(0f64, 10f64, 0f64)),
        Vector3::new(5f64, -4f64, 2f64),
        1f64,
        200,
        1f64,
    );

    assert_intersection_equals(
        None,
        intersect_ray_and_surface(&hitting_ray, &surface, 201, 205, Some(120), 0),
    );
}

#[test]
fn narrowly_hit_static_surface() {
    let surface = static_surface();

    let narrowly_hitting_ray = Ray::new(
        Unit::new_normalize(Vector3::new(0f64, 1f64, 0f64)),
        Vector3::new(0f64, 0f64, 0f64),
        1f64,
        800,
        1f64,
    );

    assert_intersection_equals(
        Some((803, Vector3::new(0f64, 3f64, 0f64))),
        intersect_ray_and_surface(&narrowly_hitting_ray, &surface, 800, 900, Some(400), 0),
    );
}

#[test]
fn narrowly_miss_static_surface() {
    let surface = static_surface();

    let narrowly_missing_ray = Ray::new(
        Unit::new_normalize(Vector3::new(0f64, 1f64, 0f64)),
        Vector3::new(-0.01f64, 0f64, 0f64),
        1f64,
        1000,
        1f64,
    );

    assert_intersection_equals(
        None,
        intersect_ray_and_surface(&narrowly_missing_ray, &surface, 1000, 1100, Some(500), 0),
    );
}

#[test]
fn clearly_miss_static_surface() {
    let surface = static_surface();

    let missing_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(1f64, 0f64, 1f64)),
        Vector3::new(15f64, 0f64, 2f64),
        1f64,
        700,
        1f64,
    );

    assert_intersection_equals(
        None,
        intersect_ray_and_surface(&missing_ray, &surface, 700, 800, Some(700), 0),
    )
}

#[test]
fn clearly_hit_moving_surface() {
    let surface = moving_surface();

    let hitting_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(0f64, 10f64, 0f64)),
        Vector3::new(1f64, -7f64, 2f64),
        1f64,
        80,
        1f64,
    );

    assert_intersection_equals(
        Some((90, Vector3::new(1f64, 3f64, 2f64))),
        intersect_ray_and_surface(&hitting_ray, &surface, 0, 100, Some(20), 0),
    );
}

#[test]
fn miss_moving_surface_because_time() {
    let surface = moving_surface();

    let hitting_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(0f64, 10f64, 0f64)),
        Vector3::new(1f64, -7f64, 2f64),
        1f64,
        60,
        1f64,
    );

    assert_intersection_equals(
        None,
        intersect_ray_and_surface(&hitting_ray, &surface, 61, 65, Some(20), 0),
    );
}

#[test]
fn hit_moving_surface_with_ray_starting_late() {
    let surface = moving_surface();

    let hitting_ray_with_later_start: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(0f64, 10f64, 0f64)),
        Vector3::new(1f64, -2f64, 2f64),
        1f64,
        45,
        1f64,
    );

    assert_intersection_equals(
        Some((50, Vector3::new(1f64, 3f64, 2f64))),
        intersect_ray_and_surface(&hitting_ray_with_later_start, &surface, 0, 100, Some(20), 0),
    );
}

#[test]
fn narrowly_miss_moving_surface() {
    let surface = moving_surface();

    let narrowly_missing_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(0f64, 10f64, 0f64)),
        Vector3::new(-0.1f64, -2f64, 2f64),
        1f64,
        65,
        1f64,
    );

    assert_intersection_equals(
        None,
        intersect_ray_and_surface(&narrowly_missing_ray, &surface, 0, 100, Some(20), 0),
    );
}

#[test]
fn clearly_miss_moving_surface() {
    let surface = moving_surface();

    let clearly_missing_ray: Ray = Ray::new(
        Unit::new_normalize(Vector3::new(1f64, 0f64, 1f64)),
        Vector3::new(1f64, -2f64, 2f64),
        1f64,
        45,
        1f64,
    );

    assert_intersection_equals(
        None,
        intersect_ray_and_surface(&clearly_missing_ray, &surface, 0, 100, Some(20), 0),
    );
}

/*
let narrowly_hitting_ray = Ray::new(Unit::new_normalize(Vector3::new(0f64, 1f64, 0f64)), Vector3::new(0f64, 0f64, 0f64), 1f64, 0, 1f64);

assert_intersection_equal(
    Some((3, Vector3::new(0f64, 3f64, 0f64))),
    intersect_ray_and_surface(&narrowly_hitting_ray, &surface, 0, 100),
);

let narrowly_missing_ray = Ray::new(Unit::new_normalize(Vector3::new(0f64, 1f64, 0f64)), Vector3::new(-0.01f64, 0f64, 0f64), 1f64, 0, 1f64);

assert_intersection_equal(
    None,
    intersect_ray_and_surface(&narrowly_missing_ray, &surface, 0, 100),
);

let missing_ray: Ray = Ray::new(Unit::new_normalize(Vector3::new(1f64, 0f64, 1f64)), Vector3::new(15f64, 0f64, 2f64), 1f64, 0, 1f64);

assert_intersection_equal(
    None,
    intersect_ray_and_surface(&missing_ray, &surface, 0, 100),
)
*/
