use demo::{
    materials::MATERIAL_CONCRETE_WALL, ray::DEFAULT_PROPAGATION_SPEED, scene_builder::SceneBuilder, scene::SceneData, DEFAULT_SAMPLE_RATE
};

fn main() {
    let scene = SceneBuilder::new()
        .with_static_cube(-10f32, -10f32, -10f32, 10f32, 10f32, 10f32, MATERIAL_CONCRETE_WALL)
        .with_emitter_at(0f32, 0f32, 2f32)
        .build();
    let scene_data = SceneData::<typenum::U10>::create_for_scene(scene);
    println!(
        "{:?}",
        scene_data.simulate_at_time(0, 100000, DEFAULT_PROPAGATION_SPEED, DEFAULT_SAMPLE_RATE)
    );
}
