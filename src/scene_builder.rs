use nalgebra::{Point3, Rotation3, Translation3, Unit, Vector3};

use crate::{
    bounce::EmissionType,
    materials::{Material, MATERIAL_CONCRETE_WALL},
    scene::{CoordinateKeyframe, Emitter, Receiver, Scene, Surface, SurfaceData, SurfaceKeyframe},
};

/// Create a static cube primitive described by the given coordinates and material.
pub fn static_cube(
    bottom_left: Vector3<f64>,
    top_right: Vector3<f64>,
    material: Material,
) -> Vec<Surface<3>> {
    cube_polygons(bottom_left, top_right)
        .iter()
        .map(|coords| Surface::Interpolated(*coords, 0, SurfaceData::new(material)))
        .collect()
}

/// Create a rotating cube primitive described by the given coordinates and material.
pub fn rotating_cube(
    bottom_left: Vector3<f64>,
    top_right: Vector3<f64>,
    rotation_origin: Vector3<f64>,
    rotation_duration: u32,
    material: Material,
) -> Vec<Surface<3>> {
    let keyframes = rotate(
        &cube_polygons(bottom_left, top_right),
        rotation_origin,
        rotation_duration,
    );
    keyframes
        .iter()
        .map(|keys| Surface::Keyframes(keys.clone(), SurfaceData::new(material)))
        .collect()
}

/// Create a static L primitive described by the given coordinates and material.
pub fn static_l(
    bottom_left: Vector3<f64>,
    length_1: f64,
    length_2: f64,
    width_1: f64,
    width_2: f64,
    height: f64,
    material: Material,
) -> Vec<Surface<3>> {
    l_polygons(bottom_left, length_1, length_2, width_1, width_2, height)
        .iter()
        .map(|coords| Surface::Interpolated(*coords, 0, SurfaceData::new(material)))
        .collect()
}

/// Create a rotating L primitive described by the given coordinates and material.
pub fn rotating_l(
    bottom_left: Vector3<f64>,
    length_1: f64,
    length_2: f64,
    width_1: f64,
    width_2: f64,
    height: f64,
    rotation_origin: Vector3<f64>,
    rotation_duration: u32,
    material: Material,
) -> Vec<Surface<3>> {
    let keyframes = rotate(
        &l_polygons(bottom_left, length_1, length_2, width_1, width_2, height),
        rotation_origin,
        rotation_duration,
    );
    keyframes
        .iter()
        .map(|keys| Surface::Keyframes(keys.clone(), SurfaceData::new(material)))
        .collect()
}

#[allow(clippy::too_many_lines)]
fn cube_polygons(bottom_left: Vector3<f64>, top_right: Vector3<f64>) -> [[Vector3<f64>; 3]; 12] {
    [
        // left
        [
            Vector3::new(bottom_left.x, bottom_left.y, bottom_left.z),
            Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
            Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
        ],
        [
            Vector3::new(bottom_left.x, top_right.y, top_right.z),
            Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
            Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
        ],
        // front
        [
            Vector3::new(bottom_left.x, bottom_left.y, bottom_left.z),
            Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
            Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
        ],
        [
            Vector3::new(top_right.x, bottom_left.y, top_right.z),
            Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
            Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
        ],
        // right
        [
            Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
            Vector3::new(top_right.x, bottom_left.y, top_right.z),
            Vector3::new(top_right.x, top_right.y, bottom_left.z),
        ],
        [
            Vector3::new(top_right.x, top_right.y, top_right.z),
            Vector3::new(top_right.x, bottom_left.y, top_right.z),
            Vector3::new(top_right.x, top_right.y, bottom_left.z),
        ],
        // back
        [
            Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
            Vector3::new(top_right.x, top_right.y, bottom_left.z),
            Vector3::new(bottom_left.x, top_right.y, top_right.z),
        ],
        [
            Vector3::new(top_right.x, top_right.y, top_right.z),
            Vector3::new(top_right.x, top_right.y, bottom_left.z),
            Vector3::new(bottom_left.x, top_right.y, top_right.z),
        ],
        // bottom
        [
            Vector3::new(bottom_left.x, bottom_left.y, bottom_left.z),
            Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
            Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
        ],
        [
            Vector3::new(top_right.x, top_right.y, bottom_left.z),
            Vector3::new(top_right.x, bottom_left.y, bottom_left.z),
            Vector3::new(bottom_left.x, top_right.y, bottom_left.z),
        ],
        // top
        [
            Vector3::new(bottom_left.x, bottom_left.y, top_right.z),
            Vector3::new(top_right.x, bottom_left.y, top_right.z),
            Vector3::new(bottom_left.x, top_right.y, top_right.z),
        ],
        [
            Vector3::new(top_right.x, top_right.y, top_right.z),
            Vector3::new(top_right.x, bottom_left.y, top_right.z),
            Vector3::new(bottom_left.x, top_right.y, top_right.z),
        ],
    ]
}

// polygons for an L-Shaped
fn l_polygons(
    bottom_point: Vector3<f64>,
    length_1: f64,
    length_2: f64,
    width_1: f64,
    width_2: f64,
    height: f64,
) -> [[Vector3<f64>; 3]; 20] {
    let bottom_points = [
        bottom_point,
        bottom_point + Vector3::new(0f64, width_1, 0f64),
        bottom_point + Vector3::new(length_1 - width_2, width_1, 0f64),
        bottom_point + Vector3::new(length_1 - width_2, length_2, 0f64),
        bottom_point + Vector3::new(length_1, length_2, 0f64),
        bottom_point + Vector3::new(length_1, 0f64, 0f64),
    ];
    let to_top = Vector3::new(0f64, 0f64, height);
    let wall_bottom = |idx1: usize, idx2: usize| {
        [
            bottom_points[idx1],
            bottom_points[idx1] + to_top,
            bottom_points[idx2],
        ]
    };
    let wall_top = |idx1: usize, idx2: usize| {
        [
            bottom_points[idx1] + to_top,
            bottom_points[idx2] + to_top,
            bottom_points[idx2],
        ]
    };
    let floor_points = [
        // first poly
        bottom_point,
        bottom_points[1],
        bottom_point + Vector3::new(length_1, width_1, 0f64),
        bottom_points[5],
        // second poly
        bottom_points[2],
        bottom_points[3],
        bottom_points[4],
        bottom_point + Vector3::new(length_1, width_1, 0f64),
    ];
    [
        wall_bottom(0, 1),
        wall_top(0, 1),
        wall_bottom(1, 2),
        wall_top(1, 2),
        wall_bottom(2, 3),
        wall_top(2, 3),
        wall_bottom(3, 4),
        wall_top(3, 4),
        wall_bottom(4, 5),
        wall_top(4, 5),
        wall_bottom(5, 1),
        wall_top(5, 1),
        [floor_points[0], floor_points[1], floor_points[2]],
        [floor_points[3], floor_points[1], floor_points[2]],
        [floor_points[4], floor_points[5], floor_points[6]],
        [floor_points[7], floor_points[5], floor_points[6]],
        [
            floor_points[0] + to_top,
            floor_points[1] + to_top,
            floor_points[2] + to_top,
        ],
        [
            floor_points[3] + to_top,
            floor_points[1] + to_top,
            floor_points[2] + to_top,
        ],
        [
            floor_points[4] + to_top,
            floor_points[5] + to_top,
            floor_points[6] + to_top,
        ],
        [
            floor_points[7] + to_top,
            floor_points[5] + to_top,
            floor_points[6] + to_top,
        ],
    ]
}

fn rotate(
    coordinates: &[[Vector3<f64>; 3]],
    rotation_origin: Vector3<f64>,
    rotation_duration: u32,
) -> Vec<Vec<SurfaceKeyframe<3>>> {
    let (number_of_keyframes, time_factor) = if rotation_duration < 1000 {
        (rotation_duration, 1)
    } else {
        (rotation_duration / 100, 100)
    };
    let from_origin = Translation3::from(rotation_origin);
    let z_axis = Unit::new_unchecked(Vector3::new(0f64, 0f64, 1f64));
    coordinates
        .iter()
        .map(|coords| {
            let point_coords: Vec<Point3<f64>> =
                coords.iter().map(|coord| Point3::from(*coord)).collect();
            (0..=number_of_keyframes)
                .map(|num| {
                    let rot_amount = f64::from(num) / f64::from(number_of_keyframes);
                    let rot = Rotation3::from_axis_angle(
                        &z_axis,
                        2f64 * std::f64::consts::PI * rot_amount,
                    );
                    let result_coords: Vec<Vector3<f64>> = point_coords
                        .iter()
                        .map(|coord| {
                            let point = from_origin.transform_point(
                                &rot.transform_point(&from_origin.inverse_transform_point(coord)),
                            );
                            let homog = point.to_homogeneous();
                            Vector3::new(homog.x / homog.w, homog.y / homog.w, homog.z / homog.w)
                        })
                        .collect();
                    SurfaceKeyframe {
                        coords: (&result_coords[0..3]).try_into().unwrap(),
                        time: time_factor * num,
                    }
                })
                .collect()
        })
        .collect()
}

/// Representations of object primitives `SceneBuilder` can create.
enum Object {
    StaticCube(Vector3<f64>, Vector3<f64>, Material),
    RotatingCube(Vector3<f64>, Vector3<f64>, Vector3<f64>, u32, Material),
    StaticL(Vector3<f64>, f64, f64, f64, f64, f64, Material),
    RotatingL(
        Vector3<f64>,
        f64,
        f64,
        f64,
        f64,
        f64,
        Vector3<f64>,
        u32,
        Material,
    ),
}

impl Object {
    fn build(&self) -> Vec<Surface<3>> {
        match self {
            Object::StaticCube(bottom_left, top_right, material) => {
                static_cube(*bottom_left, *top_right, *material)
            }
            Object::RotatingCube(
                bottom_left,
                top_right,
                rotation_origin,
                rotation_duration,
                material,
            ) => rotating_cube(
                *bottom_left,
                *top_right,
                *rotation_origin,
                *rotation_duration,
                *material,
            ),
            Object::StaticL(
                bottom_left,
                length_1,
                length_2,
                width_1,
                width_2,
                height,
                material,
            ) => static_l(
                *bottom_left,
                *length_1,
                *length_2,
                *width_1,
                *width_2,
                *height,
                *material,
            ),
            Object::RotatingL(
                bottom_left,
                length_1,
                length_2,
                width_1,
                width_2,
                height,
                rotation_origin,
                rotation_duration,
                material,
            ) => rotating_l(
                *bottom_left,
                *length_1,
                *length_2,
                *width_1,
                *width_2,
                *height,
                *rotation_origin,
                *rotation_duration,
                *material,
            ),
        }
    }
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
    loop_duration: Option<u32>,
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
        bottom_left: (f64, f64, f64),
        top_right: (f64, f64, f64),
        material: Material,
    ) -> Self {
        self.objects.push(Object::StaticCube(
            Vector3::new(bottom_left.0, bottom_left.1, bottom_left.2),
            Vector3::new(top_right.0, top_right.1, top_right.2),
            material,
        ));
        self
    }

    /// Add a rotating cube to the scene.
    #[allow(clippy::too_many_arguments)]
    pub fn with_rotating_cube(
        mut self,
        bottom_left: (f64, f64, f64),
        top_right: (f64, f64, f64),
        rotation_origin: (f64, f64, f64),
        rotation_time: u32,
        material: Material,
    ) -> Self {
        self.objects.push(Object::RotatingCube(
            Vector3::new(bottom_left.0, bottom_left.1, bottom_left.2),
            Vector3::new(top_right.0, top_right.1, top_right.2),
            Vector3::new(rotation_origin.0, rotation_origin.1, rotation_origin.2),
            rotation_time,
            material,
        ));
        self
    }

    /// Add a static cube to the scene.
    #[allow(clippy::too_many_arguments)]
    pub fn with_static_l(
        mut self,
        bottom_left: (f64, f64, f64),
        length_1: f64,
        length_2: f64,
        width_1: f64,
        width_2: f64,
        height: f64,
        material: Material,
    ) -> Self {
        self.objects.push(Object::StaticL(
            Vector3::new(bottom_left.0, bottom_left.1, bottom_left.2),
            length_1,
            length_2,
            width_1,
            width_2,
            height,
            material,
        ));
        self
    }

    /// Add a L to the scene.
    #[allow(clippy::too_many_arguments)]
    pub fn with_rotating_l(
        mut self,
        bottom_left: (f64, f64, f64),
        length_1: f64,
        length_2: f64,
        width_1: f64,
        width_2: f64,
        height: f64,
        rotation_origin: (f64, f64, f64),
        rotation_time: u32,
        material: Material,
    ) -> Self {
        self.objects.push(Object::RotatingL(
            Vector3::new(bottom_left.0, bottom_left.1, bottom_left.2),
            length_1,
            length_2,
            width_1,
            width_2,
            height,
            Vector3::new(rotation_origin.0, rotation_origin.1, rotation_origin.2),
            rotation_time,
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

    /// Set the scene to not loop.
    pub const fn non_looping(mut self) -> Self {
        self.loop_duration = None;
        self
    }

    /// Set the scene to loop with the specified duration.
    pub const fn looping(mut self, duration: u32) -> Self {
        self.loop_duration = Some(duration);
        self
    }

    /// Build the `Scene` described by the data passed into this `SceneBuilder`.
    ///
    /// # Panics
    /// * If somehow neither coordinate keyframes nor coordinates for a receiver/emitter are set. This shouldn't be able to happen.
    #[allow(clippy::option_if_let_else)]
    pub fn build(&self) -> Scene {
        let objects: Vec<Vec<Surface<3>>> = self.objects.iter().map(Object::build).collect();
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
            loop_duration: self.loop_duration,
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
            loop_duration: None,
        }
    }
}

/// A scene inside a static cube.
/// The cube is 4x4x3 meters in size.
pub fn static_cube_scene() -> Scene {
    SceneBuilder::new()
        .with_static_cube(
            (-2f64, -2f64, -1.5f64),
            (2f64, 2f64, 1.5f64),
            MATERIAL_CONCRETE_WALL,
        )
        .with_emitter_at(0f64, 0f64, 1.2f64)
        .build()
}

/// A scene inside a rotating cube.
/// The cube is 4x4x3 meters in size.
pub fn rotating_cube_scene(sample_rate: u32) -> Scene {
    SceneBuilder::new()
        .with_rotating_cube(
            (-2f64, -2f64, -1.5f64),
            (2f64, 2f64, 1.5f64),
            (0f64, 0f64, 0f64),
            sample_rate,
            MATERIAL_CONCRETE_WALL,
        )
        .with_emitter_at(0f64, 0f64, 1.2f64)
        .looping(sample_rate)
        .build()
}

/// A scene inside a rotating cube.
/// The cube is 4x4x3 meters in size.
pub fn rotating_l_scene(sample_rate: u32) -> Scene {
    SceneBuilder::new()
        .with_rotating_l(
            (-1f64, -1f64, -1f64),
            5f64,
            3f64,
            2f64,
            2f64,
            2f64,
            (0f64, 0f64, 0f64),
            sample_rate,
            MATERIAL_CONCRETE_WALL,
        )
        .with_emitter_at(0f64, 0f64, 1.2f64)
        .looping(sample_rate)
        .build()
}

/// A scene without surfaces,
/// where the receiver is exactly 1 second of travelling at the speed of sound away from the emitter.
pub fn static_receiver_scene() -> Scene {
    SceneBuilder::new()
        .with_directed_emission(1f64, 0f64, 0f64)
        .with_receiver_at(343.3f64, 0f64, 0f64)
        .build()
}

/// A scene without surfaces, where the receiver starts 1 second of speed of sound travel away
/// and approaches the emitter at 1/9th the speed of sound.
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

/// A scene without surfaces, where the receiver starts 4 seconds of speed of sound travel away
/// and approaches the emitter at 1/9th the speed of sound.
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
