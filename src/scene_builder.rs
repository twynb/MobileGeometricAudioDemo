use nalgebra::Vector3;

use crate::{
    bounce::EmissionType,
    materials::{Material, MATERIAL_CONCRETE_WALL},
    scene::{CoordinateKeyframe, Emitter, Receiver, Scene, Surface},
};

/// Create a static cube primitive described by the given coordinates and material.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn static_cube(
    bottom_left: Vector3<f64>,
    top_right: Vector3<f64>,
    material: Material,
) -> Vec<Surface<3>> {
    vec![
        // left
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
            material,
        ),
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
            material,
        ),
        // front
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, bottom_left.y, bottom_left.z),
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
            ],
            0,
            material,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
            ],
            0,
            material,
        ),
        // right
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
            ],
            0,
            material,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, top_right.y, top_right.z),
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
            ],
            0,
            material,
        ),
        // back
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
            material,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, top_right.y, top_right.z),
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
            material,
        ),
        // bottom
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, bottom_left.y, bottom_left.z),
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
            material,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, top_right.y, bottom_left.z),
                Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
                Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
            ],
            0,
            material,
        ),
        // top
        Surface::Interpolated(
            [
                Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
            material,
        ),
        Surface::Interpolated(
            [
                Vector3::new(top_right.x, top_right.y, top_right.z),
                Vector3::new(top_right.x, bottom_left.y, top_right.z),
                Vector3::new(bottom_left.x, top_right.y, top_right.z),
            ],
            0,
            material,
        ),
    ]
}

/// Representations of object primitives `SceneBuilder` can create.
enum Object {
    StaticCube(Vector3<f64>, Vector3<f64>, Material),
}

/// A builder to easily create scenes with.
pub struct SceneBuilder {
    objects: Vec<Object>,
    receiver_coords: Option<Vector3<f64>>,
    receiver_keyframes: Option<Vec<CoordinateKeyframe>>,
    receiver_radius: f64,
    emitter_coords: Option<Vector3<f64>>,
    emitter_keyframes: Option<Vec<CoordinateKeyframe>>,
    emission_type: EmissionType,
}

impl SceneBuilder {
    /// Start building a new scene.
    /// The initial scene has a receiver at (0, 0, 0) with radius 0.1,
    /// an emitter at the same position
    /// and no surfaces.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a static cube to the scene.
    #[allow(clippy::too_many_arguments)]
    pub fn with_static_cube(
        mut self,
        bottom_left_x: f64,
        bottom_left_y: f64,
        bottom_left_z: f64,
        top_right_x: f64,
        top_right_y: f64,
        top_right_z: f64,
        material: Material,
    ) -> Self {
        self.objects.push(Object::StaticCube(
            Vector3::new(bottom_left_x, bottom_left_y, bottom_left_z),
            Vector3::new(top_right_x, top_right_y, top_right_z),
            material,
        ));
        self
    }

    /// Set the coordinates for the receiver.
    /// If coordinates or coordinate keyframes have previously been set,
    /// they are discarded in favour of the new coordinates.
    pub fn with_receiver_at(mut self, x: f64, y: f64, z: f64) -> Self {
        self.receiver_coords = Some(Vector3::new(x, y, z));
        self.receiver_keyframes = None;
        self
    }

    /// Set the coordinate keyframes for the receiver.
    /// If coordinates or coordinate keyframes have previously been set,
    /// they are discarded in favour of the new coordinate keyframes.
    pub fn with_receiver_keyframes(mut self, coords: Vec<CoordinateKeyframe>) -> Self {
        self.receiver_keyframes = Some(coords);
        self.receiver_coords = None;
        self
    }

    /// Set the radius for the receiver.
    pub const fn with_receiver_radius(mut self, radius: f64) -> Self {
        self.receiver_radius = radius;
        self
    }

    /// Set the coordinates for the emitter.
    /// If coordinates or coordinate keyframes have previously been set,
    /// they are discarded in favour of the new coordinates.
    pub fn with_emitter_at(mut self, x: f64, y: f64, z: f64) -> Self {
        self.emitter_coords = Some(Vector3::new(x, y, z));
        self.emitter_keyframes = None;
        self
    }

    /// Set the coordinate keyframes for the emitter.
    /// If coordinates or coordinate keyframes have previously been set,
    /// they are discarded in favour of the new coordinate keyframes.
    pub fn with_emitter_keyframes(mut self, coords: Vec<CoordinateKeyframe>) -> Self {
        self.emitter_keyframes = Some(coords);
        self.emitter_coords = None;
        self
    }

    /// Set the emission type to be randomised, i.e. rays are initially launched in all directions.
    pub const fn with_random_emission(mut self) -> Self {
        self.emission_type = EmissionType::Random;
        self
    }

    /// Set the emission type to have a specific direction, i.e. all rays are initially launched in this direction.
    pub fn with_directed_emission(mut self, x: f64, y: f64, z: f64) -> Self {
        self.emission_type = EmissionType::Directed(Vector3::new(x, y, z).normalize());
        self
    }

    /// Build the `Scene` described by the data passed into this `SceneBuilder`.
    ///
    /// # Panics
    /// * If somehow neither coordinate keyframes nor coordinates for a receiver/emitter are set. This shouldn't be able to happen.
    #[allow(clippy::option_if_let_else)]
    pub fn build(&self) -> Scene {
        let objects: Vec<Vec<Surface<3>>> = self
            .objects
            .iter()
            .map(|object| match object {
                Object::StaticCube(bottom_left, top_right, material) => {
                    static_cube(*bottom_left, *top_right, *material)
                }
            })
            .collect();
        let mut surfaces: Vec<Surface<3>> = Vec::with_capacity(objects.len() * 6);
        for object in &objects {
            surfaces.extend_from_slice(object);
        }

        let receiver = if let Some(coords) = self.receiver_coords {
            Receiver::Interpolated(coords, self.receiver_radius, 0)
        } else if let Some(keyframes) = &self.receiver_keyframes {
            Receiver::Keyframes(keyframes.clone(), self.receiver_radius)
        } else {
            panic!("Somehow, neither receiver_keyframes nor receiver_coords was set. This shouldn't happen.")
        };

        let emitter = if let Some(coords) = self.emitter_coords {
            Emitter::Interpolated(coords, 0, self.emission_type)
        } else if let Some(keyframes) = &self.emitter_keyframes {
            Emitter::Keyframes(keyframes.clone(), self.emission_type)
        } else {
            panic!("Somehow, neither emitter_keyframes nor emitter_coords was set. This shouldn't happen.")
        };

        Scene {
            surfaces,
            receiver,
            emitter,
        }
    }
}

impl Default for SceneBuilder {
    fn default() -> Self {
        Self {
            objects: vec![],
            receiver_coords: Some(Vector3::new(0f64, 0f64, 0f64)),
            receiver_keyframes: None,
            receiver_radius: 0.1f64,
            emitter_coords: Some(Vector3::new(0f64, 0f64, 0f64)),
            emitter_keyframes: None,
            emission_type: EmissionType::Random,
        }
    }
}

pub fn static_cube_scene() -> Scene {
    SceneBuilder::new()
        .with_static_cube(
            -10f64,
            -10f64,
            -10f64,
            10f64,
            10f64,
            10f64,
            MATERIAL_CONCRETE_WALL,
        )
        .with_emitter_at(0f64, 0f64, 1.2f64)
        .build()
}

pub fn static_receiver_scene() -> Scene {
    SceneBuilder::new()
        .with_directed_emission(1f64, 0f64, 0f64)
        .with_receiver_at(343.3f64, 0f64, 0f64)
        .build()
}

pub fn approaching_receiver_scene(sample_rate: u32) -> Scene {
    SceneBuilder::new()
        .with_directed_emission(1f64, 0f64, 0f64)
        .with_receiver_keyframes(vec![
            CoordinateKeyframe {
                coords: Vector3::new(343.3f64, 0f64, 0f64),
                time: 0,
            },
            CoordinateKeyframe {
                coords: Vector3::new(0f64, 0f64, 0f64),
                time: sample_rate * 9,
            },
        ])
        .build()
}


pub fn long_approaching_receiver_scene(sample_rate: u32) -> Scene {
    SceneBuilder::new()
        .with_directed_emission(1f64, 0f64, 0f64)
        .with_receiver_keyframes(vec![
            CoordinateKeyframe {
                coords: Vector3::new(343.3f64 * 4f64, 0f64, 0f64),
                time: 0,
            },
            CoordinateKeyframe {
                coords: Vector3::new(0f64, 0f64, 0f64),
                time: sample_rate * 9 * 4,
            },
        ])
        .build()
}
