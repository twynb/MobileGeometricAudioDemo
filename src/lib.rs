/// The default sample rate of 44.1 `KHz`.
pub const DEFAULT_SAMPLE_RATE: f64 = 44100f64;

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
pub mod impulse_response;
pub mod bounce;