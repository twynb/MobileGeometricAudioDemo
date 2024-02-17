#![allow(dead_code)]
// remove this once integrating - this is to avoid exessive and useless warnings for the time being

use std::ops::Mul;

use generic_array::ArrayLength;
use interpolation::Interpolation;
use nalgebra::Vector3;
use rand::random;
use ray::Ray;
use rayon::prelude::*;
use scene::{Emitter, Scene};
use scene_bounds::MaximumBounds;
use typenum::Unsigned;

pub const DEFAULT_SAMPLE_RATE: f32 = 44100f32;

pub mod chunk;
pub mod interpolation;
pub mod intersection;
pub mod materials;
mod maths;
pub mod ray;
pub mod scene;
pub mod scene_bounds;
pub mod scene_builder;
mod test_utils;

/// General data about a scene, required to bounce a ray through.
/// Contains the scene itself, its maximum boundaries and its
/// chunk representation.
pub struct SceneData<C: Unsigned>
where
    C: Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    pub scene: scene::Scene,
    pub chunks: chunk::Chunks<C>,
    pub maximum_bounds: (nalgebra::Vector3<f32>, nalgebra::Vector3<f32>),
}

impl<C: Unsigned> SceneData<C>
where
    C: Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    pub fn create_for_scene(scene: Scene) -> Self {
        let chunks = scene.chunks::<C>();
        let maximum_bounds = scene.maximum_bounds();
        Self {
            scene,
            chunks,
            maximum_bounds
        }
    }

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

    fn launch_ray(&self, time: u32, velocity: f32, sample_rate: f32) -> Vec<(f32, u32)> {
        let Emitter::Interpolated(emitter_coords, _) = self.scene.emitter.at_time(time) else {
            todo!()
        };
        Ray::launch(
            // doesn't need to be a unit vector, Ray::launch() normalises this
            Vector3::new(
                random::<f32>() * 2f32 - 1f32,
                random::<f32>() * 2f32 - 1f32,
                random::<f32>() * 2f32 - 1f32,
            ),
            emitter_coords,
            time,
            velocity,
            sample_rate,
            self,
        )
    }
}
