use demo::{
    bounce::EmissionType,
    materials::{Material, MATERIAL_CONCRETE_WALL},
    ray::{Ray, DEFAULT_PROPAGATION_SPEED},
    scene::{Emitter, Receiver, Scene, SceneData, Surface, SurfaceData},
    scene_bounds::MaximumBounds,
    scene_builder, DEFAULT_SAMPLE_RATE,
};
use nalgebra::Vector3;

#[test]
fn directly_hitting_receiver() {
    let scene = Scene {
        surfaces: vec![Surface::Interpolated(
            [
                Vector3::new(-10f64, 10f64, -10f64),
                Vector3::new(40f64, 10f64, -10f64),
                Vector3::new(-10f64, 10f64, 40f64),
            ],
            0,
            SurfaceData::new(Material {
                absorption_coefficient: 0.9,
                diffusion_coefficient: 0f64,
            }),
        )],
        receiver: Receiver::Interpolated(Vector3::new(20f64, 0f64, 0f64), 0.1f64, 0),
        emitter: Emitter::Interpolated(Vector3::new(0f64, 0f64, 0f64), 0, EmissionType::Random),
        loop_duration: None,
    };
    let chunks = scene.chunks::<typenum::U10>();
    let maximum_bounds = scene.maximum_bounds();
    let scene_data = SceneData {
        scene,
        chunks,
        maximum_bounds,
    };
    let direction = Vector3::new(1f64, 0f64, 0f64);
    let result = Ray::launch(
        direction,
        Vector3::new(0f64, 0f64, 0f64),
        0,
        DEFAULT_PROPAGATION_SPEED,
        DEFAULT_SAMPLE_RATE,
        &scene_data,
    );

    let expected = vec![(1f64, 2557u32)];
    assert_eq!(expected, result);
}

#[test]
fn hitting_receiver_after_one_bounce() {
    let scene = Scene {
        surfaces: vec![Surface::Interpolated(
            [
                Vector3::new(-10f64, 10f64, -10f64),
                Vector3::new(-10f64, 10f64, 40f64),
                Vector3::new(40f64, 10f64, -10f64),
            ],
            0,
            SurfaceData::new(Material {
                absorption_coefficient: 0.9,
                diffusion_coefficient: 0f64,
            }),
        )],
        receiver: Receiver::Interpolated(Vector3::new(20f64, 0f64, 0f64), 0.1f64, 0),
        emitter: Emitter::Interpolated(Vector3::new(0f64, 0f64, 0f64), 0, EmissionType::Random),
        loop_duration: None,
    };
    let chunks = scene.chunks::<typenum::U10>();
    let maximum_bounds = scene.maximum_bounds();
    let scene_data = SceneData {
        scene,
        chunks,
        maximum_bounds,
    };
    let direction = Vector3::new(1f64, 1f64, 0f64);
    let result = Ray::launch(
        direction,
        Vector3::new(0f64, 0f64, 0f64),
        0,
        DEFAULT_PROPAGATION_SPEED,
        DEFAULT_SAMPLE_RATE,
        &scene_data,
    );

    let expected = vec![(0.9f64, 3622u32)];
    assert_eq!(expected, result);
}

#[test]
fn unreachable_receiver() {
    let scene = Scene {
        surfaces: scene_builder::static_cube(
            Vector3::new(-5f64, -5f64, -5f64),
            Vector3::new(-5f64, -5f64, -5f64),
            MATERIAL_CONCRETE_WALL,
        ),
        receiver: Receiver::Interpolated(Vector3::new(20f64, 0f64, 0f64), 0.1f64, 0),
        emitter: Emitter::Interpolated(Vector3::new(0f64, 0f64, 0f64), 0, EmissionType::Random),
        loop_duration: None,
    };
    let chunks = scene.chunks::<typenum::U10>();
    let maximum_bounds = scene.maximum_bounds();
    let scene_data = SceneData {
        scene,
        chunks,
        maximum_bounds,
    };
    let direction = Vector3::new(1f64, 1f64, 0f64);
    let result = Ray::launch(
        direction,
        Vector3::new(0f64, 0f64, 0f64),
        0,
        DEFAULT_PROPAGATION_SPEED,
        DEFAULT_SAMPLE_RATE,
        &scene_data,
    );

    let expected: Vec<(f64, u32)> = vec![];
    assert_eq!(expected, result);
}

#[test]
fn hitting_receiver_before_and_after_one_bounce() {
    let scene = Scene {
        surfaces: vec![Surface::Interpolated(
            [
                Vector3::new(40f64, -10f64, -10f64),
                Vector3::new(40f64, 40f64, -10f64),
                Vector3::new(40f64, -100f64, 40f64),
            ],
            0,
            SurfaceData::new(Material {
                absorption_coefficient: 0.9,
                diffusion_coefficient: 0f64,
            }),
        )],
        receiver: Receiver::Interpolated(Vector3::new(20f64, 0f64, 0f64), 0.1f64, 0),
        emitter: Emitter::Interpolated(Vector3::new(0f64, 0f64, 0f64), 0, EmissionType::Random),
        loop_duration: None,
    };
    let chunks = scene.chunks::<typenum::U10>();
    let maximum_bounds = scene.maximum_bounds();
    let scene_data = SceneData {
        scene,
        chunks,
        maximum_bounds,
    };
    let direction = Vector3::new(1f64, 0f64, 0f64);
    let result = Ray::launch(
        direction,
        Vector3::new(0f64, 0f64, 0f64),
        0,
        DEFAULT_PROPAGATION_SPEED,
        DEFAULT_SAMPLE_RATE,
        &scene_data,
    );

    let expected = vec![(1.0f64, 2557u32), (0.9f64, 7697u32)];
    assert_eq!(expected, result);
}

#[test]
fn not_hitting_receiver_behind_ray() {
    let scene = Scene {
        surfaces: vec![
            Surface::Interpolated(
                [
                    Vector3::new(-10f64, 10f64, -10f64),
                    Vector3::new(40f64, 10f64, -10f64),
                    Vector3::new(-10f64, 10f64, 40f64),
                ],
                0,
                SurfaceData::new(Material {
                    absorption_coefficient: 0.9,
                    diffusion_coefficient: 0f64,
                }),
            ),
            Surface::Interpolated(
                [
                    Vector3::new(-10f64, -10f64, -10f64),
                    Vector3::new(40f64, -10f64, -10f64),
                    Vector3::new(-10f64, -10f64, 40f64),
                ],
                0,
                SurfaceData::new(Material {
                    absorption_coefficient: 0.9,
                    diffusion_coefficient: 0f64,
                }),
            ),
        ],
        receiver: Receiver::Interpolated(Vector3::new(-20f64, 0f64, 0f64), 0.1f64, 0),
        emitter: Emitter::Interpolated(Vector3::new(0f64, 0f64, 0f64), 0, EmissionType::Random),
        loop_duration: None,
    };
    let chunks = scene.chunks::<typenum::U10>();
    let maximum_bounds = scene.maximum_bounds();
    let scene_data = SceneData {
        scene,
        chunks,
        maximum_bounds,
    };
    let direction = Vector3::new(1f64, 0f64, 0f64);
    let result = Ray::launch(
        direction,
        Vector3::new(0f64, 0f64, 0f64),
        0,
        DEFAULT_PROPAGATION_SPEED,
        DEFAULT_SAMPLE_RATE,
        &scene_data,
    );

    let expected: Vec<(f64, u32)> = vec![];
    assert_eq!(expected, result);
}

#[test]
fn not_hitting_receiver_behind_ray_reverse() {
    let scene = Scene {
        surfaces: vec![
            Surface::Interpolated(
                [
                    Vector3::new(-10f64, 10f64, -10f64),
                    Vector3::new(40f64, 10f64, -10f64),
                    Vector3::new(-10f64, 10f64, 40f64),
                ],
                0,
                SurfaceData::new(Material {
                    absorption_coefficient: 0.9,
                    diffusion_coefficient: 0f64,
                }),
            ),
            Surface::Interpolated(
                [
                    Vector3::new(-10f64, -10f64, -10f64),
                    Vector3::new(40f64, -10f64, -10f64),
                    Vector3::new(-10f64, -10f64, 40f64),
                ],
                0,
                SurfaceData::new(Material {
                    absorption_coefficient: 0.9,
                    diffusion_coefficient: 0f64,
                }),
            ),
        ],
        receiver: Receiver::Interpolated(Vector3::new(20f64, 0f64, 0f64), 0.1f64, 0),
        emitter: Emitter::Interpolated(Vector3::new(0f64, 0f64, 0f64), 0, EmissionType::Random),
        loop_duration: None,
    };
    let chunks = scene.chunks::<typenum::U10>();
    let maximum_bounds = scene.maximum_bounds();
    let scene_data = SceneData {
        scene,
        chunks,
        maximum_bounds,
    };
    let direction = Vector3::new(-1f64, 0f64, 0f64);
    let result = Ray::launch(
        direction,
        Vector3::new(0f64, 0f64, 0f64),
        0,
        DEFAULT_PROPAGATION_SPEED,
        DEFAULT_SAMPLE_RATE,
        &scene_data,
    );

    let expected: Vec<(f64, u32)> = vec![];
    assert_eq!(expected, result);
}
