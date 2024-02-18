#![allow(dead_code)]
// remove this once integrating - this is to avoid exessive and useless warnings for the time being

/// The default sample rate of 44.1 `KHz`.
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
