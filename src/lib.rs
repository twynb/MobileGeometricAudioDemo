#![allow(dead_code)]
// remove this once integrating - this is to avoid exessive and useless warnings for the time being

use std::ops::Mul;

use generic_array::ArrayLength;
use typenum::Unsigned;

pub const DEFAULT_SAMPLE_RATE: f32 = 44100f32;

pub mod chunk;
pub mod interpolation;
pub mod intersection;
pub mod ray;
pub mod scene;
pub mod scene_bounds;
pub mod scene_builder;
mod test_utils;
mod maths;
pub mod materials;

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
    pub maximum_bounds: (nalgebra::Vector3<f32>, nalgebra::Vector3<f32>)
}
