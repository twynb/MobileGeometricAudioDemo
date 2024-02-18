use std::ops::Mul;

use generic_array::ArrayLength;
use nalgebra::Vector3;
use rayon::prelude::*;
use typenum::Unsigned;
use wav::BitDepth;

use crate::{
    bounce::EmissionType,
    chunk::Chunks,
    impulse_response::{self, to_impulse_response},
    interpolation::Interpolation,
    materials::Material,
    ray::Ray,
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
    Keyframes(Vec<CoordinateKeyframe>, EmissionType),
    Interpolated(Vector3<f32>, u32, EmissionType),
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

    /// Simulate the given number of rays in this `Scene` for each sample in the given time span,
    /// then collect all the impulse responses.
    pub fn simulate_for_time_span(
        &self,
        input_data: &BitDepth,
        number_of_rays: u32,
        velocity: f32,
        sample_rate: f32,
        scaling_factor: f32,
    ) -> BitDepth {
        match input_data {
            BitDepth::Eight(data) => BitDepth::Eight(self.simulate_for_time_span_internal(
                data,
                number_of_rays,
                velocity,
                sample_rate,
                scaling_factor,
            )),
            BitDepth::Sixteen(data) => BitDepth::Sixteen(self.simulate_for_time_span_internal(
                data,
                number_of_rays,
                velocity,
                sample_rate,
                scaling_factor,
            )),
            BitDepth::TwentyFour(data) => {
                BitDepth::TwentyFour(self.simulate_for_time_span_internal(
                    data,
                    number_of_rays,
                    velocity,
                    sample_rate,
                    scaling_factor,
                ))
            }
            BitDepth::ThirtyTwoFloat(data) => {
                BitDepth::ThirtyTwoFloat(self.simulate_for_time_span_internal(
                    data,
                    number_of_rays,
                    velocity,
                    sample_rate,
                    scaling_factor,
                ))
            }
            BitDepth::Empty => BitDepth::Empty,
        }
    }

    fn simulate_for_time_span_internal<
        T: num::Num + num::NumCast + Clone + Copy + Sync + Send + std::fmt::Debug,
    >(
        &self,
        data: &[T],
        number_of_rays: u32,
        velocity: f32,
        sample_rate: f32,
        scaling_factor: f32,
    ) -> Vec<T> {
        let buffers: Vec<Vec<f32>> = data
            .iter()
            .enumerate()
            .map(|(idx, val)| (idx, *val))
            .collect::<Vec<(usize, T)>>()
            .par_chunks(1000)
            .map(|chunk| {
                println!("{}", chunk[0].0);
                self.simulate_for_chunk(
                    data.len(),
                    chunk,
                    number_of_rays,
                    velocity,
                    sample_rate,
                    scaling_factor,
                )
            })
            .collect();
        let max_len = buffers.iter().max_by_key(|vec| vec.len()).unwrap().len();
        let mut buffer = vec![0f32; max_len];
        for buffer_to_add in &buffers {
            buffer
                .iter_mut()
                .zip(buffer_to_add)
                .for_each(|(val, to_add)| *val += *to_add);
        }
        buffer
            .iter()
            .map(|val| num::cast::<f32, T>(*val).unwrap())
            .collect()
    }

    fn simulate_for_chunk<T: num::Num + num::NumCast + Clone + Copy + Sync + Send>(
        &self,
        data_len: usize,
        chunk: &[(usize, T)],
        number_of_rays: u32,
        velocity: f32,
        sample_rate: f32,
        scaling_factor: f32,
    ) -> Vec<f32> {
        let mut buffer: Vec<f32> = vec![0f32; data_len];
        for (idx, value) in chunk {
            let impulse_response =
                self.simulate_at_time(*idx as u32, number_of_rays, velocity, sample_rate);
            let buffer_to_add =
                impulse_response::apply_to_sample(&impulse_response, *value, *idx, scaling_factor);
            if buffer.len() < buffer_to_add.len() {
                buffer.resize(buffer_to_add.len(), 0f32);
            }
            buffer
                .iter_mut()
                .zip(&buffer_to_add)
                .for_each(|(val, to_add)| *val += *to_add);
        }
        buffer
    }

    /// Simulate the given number of rays at the given time in this `Scene`,
    /// then collect all the impulse responses.
    pub fn simulate_at_time(
        &self,
        time: u32,
        number_of_rays: u32,
        velocity: f32,
        sample_rate: f32,
    ) -> Vec<f32> {
        let rt_results: Vec<(f32, u32)> = (0..number_of_rays)
            .flat_map(|_| self.launch_ray(time, velocity, sample_rate))
            .collect();
        to_impulse_response(&rt_results, number_of_rays)
    }

    /// Launch a single ray into this `Scene`, and return its result.
    /// The direction it is launched in is a random position in the unit cube,
    /// which gets normalised in the ray's launch function.
    fn launch_ray(&self, time: u32, velocity: f32, sample_rate: f32) -> Vec<(f32, u32)> {
        let Emitter::Interpolated(emitter_coords, _, emission_type) = self.scene.emitter.at_time(time) else {
            // this should not be able to happen
            return vec![];
        };
        Ray::launch(
            // doesn't need to be a unit vector, Ray::launch() normalises this
            emission_type.get_direction(),
            emitter_coords,
            time,
            velocity,
            sample_rate,
            self,
        )
    }
}
