use std::ops::Mul;

use generic_array::ArrayLength;
use nalgebra::Vector3;
use rand::random;
use rayon::prelude::*;
use typenum::Unsigned;

use crate::{
    chunk::Chunks, interpolation::Interpolation, materials::Material, ray::Ray,
    scene_bounds::MaximumBounds,
};

/// Keyframe for a single set of coordinates.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct CoordinateKeyframe {
    pub time: u32,
    pub coords: Vector3<f32>,
}

/// Sound emitter.
/// Either has its separate keyframes (sorted by time) or a single interpolated keyframe at a given time.
#[derive(Clone, PartialEq, Debug)]
pub enum Emitter {
    Keyframes(Vec<CoordinateKeyframe>),
    Interpolated(Vector3<f32>, u32),
}

/// Sound receiver.
/// Either has its separate keyframes (sorted by time) or a single interpolated keyframe at a given time.
/// Always also has a radius.
#[derive(Clone, PartialEq, Debug)]
pub enum Receiver {
    Keyframes(Vec<CoordinateKeyframe>, f32),
    Interpolated(Vector3<f32>, f32, u32),
}

/// Keyframe for a set of coordinates for a surface.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SurfaceKeyframe<const N: usize> {
    pub time: u32,
    pub coords: [Vector3<f32>; N],
}

/// Surface in the scene.
/// Either has its separate keyframes (sorted by time) or a single interpolated keyframe at a given time.
/// Also contains the surface's material.
#[derive(Clone, PartialEq, Debug)]
pub enum Surface<const N: usize> {
    Keyframes(Vec<SurfaceKeyframe<N>>, Material),
    Interpolated([Vector3<f32>; N], u32, Material),
}

impl<const N: usize> Surface<N> {
    /// Calculate this surface's normal as a unit vector.
    ///
    /// # Panics
    ///
    /// * When attempting to calculate the normal on a non-interpolated surface.
    pub fn normal(&self) -> Vector3<f32> {
        match self {
            Self::Interpolated(coords, _time, _material) => {
                let cross = (coords[1] - coords[0]).cross(&(coords[2] - coords[0]));
                cross / cross.norm()
            }
            Self::Keyframes(_, _material) => {
                panic!("Normals can only be calculated for interpolated surfaces!")
            }
        }
    }
}

/// The full scene.
/// Scenes always have a single emitter and receiver, but support multiple surfaces.
#[derive(Clone, PartialEq, Debug)]
pub struct Scene {
    pub surfaces: Vec<Surface<3>>, // for now we only work with triangles
    pub receiver: Receiver,
    pub emitter: Emitter,
}

/// General data about a scene, required to bounce a ray through.
/// Contains the scene itself, its maximum boundaries and its
/// chunk representation.
#[allow(clippy::module_name_repetitions)]
pub struct SceneData<C>
where
    C: Unsigned + Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    pub scene: Scene,
    pub chunks: Chunks<C>,
    pub maximum_bounds: (nalgebra::Vector3<f32>, nalgebra::Vector3<f32>),
}

impl<C> SceneData<C>
where
    C: Unsigned + Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    /// Calculate the chunks and maximum bounds for a given `Scene`,
    /// then represent it all in a single `SceneData` object.
    /// To avoid errors, the maximum bounds are expanded by 0.1 in each direction.
    pub fn create_for_scene(scene: Scene) -> Self {
        let chunks = scene.chunks::<C>();
        let mut maximum_bounds = scene.maximum_bounds();
        maximum_bounds.0.add_scalar_mut(-0.1);
        maximum_bounds.1.add_scalar_mut(0.1);
        Self {
            scene,
            chunks,
            maximum_bounds,
        }
    }

    /// Simulate the given number of rays at the given time in this `Scene`,
    /// then collect all the impulse responses.
    pub fn simulate_at_time(
        &self,
        time: u32,
        number_of_rays: u32,
        velocity: f32,
        sample_rate: f32,
    ) -> Vec<(f32, u32)> {
        (0..number_of_rays)
            .into_par_iter()
            .flat_map(|_| self.launch_ray(time, velocity, sample_rate))
            .collect()
    }

    /// Launch a single ray into this `Scene`, and return its result.
    /// The direction it is launched in is a random position in the unit cube,
    /// which gets normalised in the ray's launch function.
    fn launch_ray(&self, time: u32, velocity: f32, sample_rate: f32) -> Vec<(f32, u32)> {
        let Emitter::Interpolated(emitter_coords, _) = self.scene.emitter.at_time(time) else {
            // this should not be able to happen
            return vec![];
        };
        Ray::launch(
            // doesn't need to be a unit vector, Ray::launch() normalises this
            Vector3::new(
                random::<f32>().mul_add(2f32, -1f32),
                random::<f32>().mul_add(2f32, -1f32),
                random::<f32>().mul_add(2f32, -1f32),
            ),
            emitter_coords,
            time,
            velocity,
            sample_rate,
            self,
        )
    }
}
