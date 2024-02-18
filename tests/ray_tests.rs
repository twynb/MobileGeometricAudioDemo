use demo::{
    materials::{Material, MATERIAL_CONCRETE_WALL},
    ray::{Ray, DEFAULT_PROPAGATION_SPEED},
    scene::{Emitter, Receiver, Scene, SceneData, Surface},
    scene_bounds::MaximumBounds,
    scene_builder, DEFAULT_SAMPLE_RATE,
};
use nalgebra::Vector3;

#[test]
fn directly_hitting_receiver() {
    let scene = Scene {
        surfaces: vec![Surface::Interpolated(
            [
                Vector3::new(-10f32, 10f32, -10f32),
                Vector3::new(40f32, 10f32, -10f32),
                Vector3::new(-10f32, 10f32, 40f32),
            ],
            0,
            Material {
                absorption_coefficient: 0.9,
                diffusion_coefficient: 0f32,
            },
        )],
        receiver: Receiver::Interpolated(Vector3::new(20f32, 0f32, 0f32), 0.1f32, 0),
        emitter: Emitter::Interpolated(Vector3::new(0f32, 0f32, 0f32), 0),
    };
    let chunks = scene.chunks::<typenum::U10>();
    let maximum_bounds = scene.maximum_bounds();
    let scene_data = SceneData {
        scene,
        chunks,
        maximum_bounds,
    };
    let direction = Vector3::new(1f32, 0f32, 0f32);
    let result = Ray::launch(
        direction,
        Vector3::new(0f32, 0f32, 0f32),
        0,
        DEFAULT_PROPAGATION_SPEED,
        DEFAULT_SAMPLE_RATE,
        &scene_data,
    );

    let expected = vec![(1f32, 2557u32)];
    assert_eq!(expected, result);
}

#[test]
fn hitting_receiver_after_one_bounce() {
    let scene = Scene {
        surfaces: vec![Surface::Interpolated(
            [
                Vector3::new(-10f32, 10f32, -10f32),
                Vector3::new(40f32, 10f32, -10f32),
                Vector3::new(-10f32, 10f32, 40f32),
            ],
            0,
            Material {
                absorption_coefficient: 0.9,
                diffusion_coefficient: 0f32,
            },
        )],
        receiver: Receiver::Interpolated(Vector3::new(20f32, 0f32, 0f32), 0.1f32, 0),
        emitter: Emitter::Interpolated(Vector3::new(0f32, 0f32, 0f32), 0),
    };
    let chunks = scene.chunks::<typenum::U10>();
    let maximum_bounds = scene.maximum_bounds();
    let scene_data = SceneData {
        scene,
        chunks,
        maximum_bounds,
    };
    let direction = Vector3::new(1f32, 1f32, 0f32);
    let result = Ray::launch(
        direction,
        Vector3::new(0f32, 0f32, 0f32),
        0,
        DEFAULT_PROPAGATION_SPEED,
        DEFAULT_SAMPLE_RATE,
        &scene_data,
    );

    let expected = vec![(0.9f32, 3621u32)];
    assert_eq!(expected, result);
}

#[test]
fn unreachable_receiver() {
    let scene = Scene {
        surfaces: scene_builder::static_cube(
            Vector3::new(-5f32, -5f32, -5f32),
            Vector3::new(-5f32, -5f32, -5f32),
            MATERIAL_CONCRETE_WALL,
        ),
        receiver: Receiver::Interpolated(Vector3::new(20f32, 0f32, 0f32), 0.1f32, 0),
        emitter: Emitter::Interpolated(Vector3::new(0f32, 0f32, 0f32), 0),
    };
    let chunks = scene.chunks::<typenum::U10>();
    let maximum_bounds = scene.maximum_bounds();
    let scene_data = SceneData {
        scene,
        chunks,
        maximum_bounds,
    };
    let direction = Vector3::new(1f32, 1f32, 0f32);
    let result = Ray::launch(
        direction,
        Vector3::new(0f32, 0f32, 0f32),
        0,
        DEFAULT_PROPAGATION_SPEED,
        DEFAULT_SAMPLE_RATE,
        &scene_data,
    );

    let expected: Vec<(f32, u32)> = vec![];
    assert_eq!(expected, result);
}

#[test]
fn hitting_receiver_before_and_after_one_bounce() {
    let scene = Scene {
        surfaces: vec![Surface::Interpolated(
            [
                Vector3::new(40f32, -10f32, -10f32),
                Vector3::new(40f32, 40f32, -10f32),
                Vector3::new(40f32, -100f32, 40f32),
            ],
            0,
            Material {
                absorption_coefficient: 0.9,
                diffusion_coefficient: 0f32,
            },
        )],
        receiver: Receiver::Interpolated(Vector3::new(20f32, 0f32, 0f32), 0.1f32, 0),
        emitter: Emitter::Interpolated(Vector3::new(0f32, 0f32, 0f32), 0),
    };
    let chunks = scene.chunks::<typenum::U10>();
    let maximum_bounds = scene.maximum_bounds();
    let scene_data = SceneData {
        scene,
        chunks,
        maximum_bounds,
    };
    let direction = Vector3::new(1f32, 0f32, 0f32);
    let result = Ray::launch(
        direction,
        Vector3::new(0f32, 0f32, 0f32),
        0,
        DEFAULT_PROPAGATION_SPEED,
        DEFAULT_SAMPLE_RATE,
        &scene_data,
    );

    let expected = vec![(1.0f32, 2557u32), (0.9f32, 5140u32)];
    assert_eq!(expected, result);
}
