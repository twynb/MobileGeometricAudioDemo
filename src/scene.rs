use std::{
    ops::Mul,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use generic_array::ArrayLength;
use nalgebra::Vector3;
use num::{Bounded, Num, NumCast};
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
    pub coords: Vector3<f64>,
}

/// Sound emitter.
/// Either has its separate keyframes (sorted by time) or a single interpolated keyframe at a given time.
#[derive(Clone, PartialEq, Debug)]
pub enum Emitter {
    Keyframes(Vec<CoordinateKeyframe>, EmissionType),
    Interpolated(Vector3<f64>, u32, EmissionType),
}

/// Sound receiver.
/// Either has its separate keyframes (sorted by time) or a single interpolated keyframe at a given time.
/// Always also has a radius.
#[derive(Clone, PartialEq, Debug)]
pub enum Receiver {
    Keyframes(Vec<CoordinateKeyframe>, f64),
    Interpolated(Vector3<f64>, f64, u32),
}

/// Keyframe for a set of coordinates for a surface.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SurfaceKeyframe<const N: usize> {
    pub time: u32,
    pub coords: [Vector3<f64>; N],
}

/// Surface in the scene.
/// Either has its separate keyframes (sorted by time) or a single interpolated keyframe at a given time.
/// Also contains the surface's material.
#[derive(Clone, PartialEq, Debug)]
pub enum Surface<const N: usize> {
    Keyframes(Vec<SurfaceKeyframe<N>>, Material),
    Interpolated([Vector3<f64>; N], u32, Material),
}

impl<const N: usize> Surface<N> {
    /// Calculate this surface's normal as a unit vector.
    ///
    /// # Panics
    ///
    /// * When attempting to calculate the normal on a non-interpolated surface.
    pub fn normal(&self) -> Vector3<f64> {
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
    pub maximum_bounds: (nalgebra::Vector3<f64>, nalgebra::Vector3<f64>),
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

    /// Simulate the given number of rays in this `Scene` for each sample in the given input,
    /// then apply the impulse response.
    /// see `simulate_for_time_span_internal` for details
    #[allow(clippy::too_many_arguments)]
    pub fn simulate_for_time_span(
        &self,
        input_data: &BitDepth,
        number_of_rays: u32,
        velocity: f64,
        sample_rate: f64,
        scaling_factor: f64,
        do_snapshot_method: bool,
        progress_counter: &Arc<AtomicU32>,
    ) -> BitDepth {
        match input_data {
            BitDepth::Eight(data) => BitDepth::Eight(self.simulate_for_time_span_internal(
                data,
                number_of_rays,
                velocity,
                sample_rate,
                scaling_factor,
                do_snapshot_method,
                progress_counter,
            )),
            BitDepth::Sixteen(data) => BitDepth::Sixteen(self.simulate_for_time_span_internal(
                data,
                number_of_rays,
                velocity,
                sample_rate,
                scaling_factor,
                do_snapshot_method,
                progress_counter,
            )),
            BitDepth::TwentyFour(data) => {
                BitDepth::TwentyFour(self.simulate_for_time_span_internal(
                    data,
                    number_of_rays,
                    velocity,
                    sample_rate,
                    scaling_factor,
                    do_snapshot_method,
                    progress_counter,
                ))
            }
            BitDepth::ThirtyTwoFloat(data) => {
                BitDepth::ThirtyTwoFloat(self.simulate_for_time_span_internal(
                    data,
                    number_of_rays,
                    velocity,
                    sample_rate,
                    scaling_factor,
                    do_snapshot_method,
                    progress_counter,
                ))
            }
            BitDepth::Empty => BitDepth::Empty,
        }
    }

    /// Simulate the scene's impulse response for each data point,
    /// then apply it to the relevant data point and collect the full result afterwards.
    /// Processing is done in chunks.
    #[allow(clippy::too_many_arguments)]
    fn simulate_for_time_span_internal<T: Num + NumCast + Clone + Copy + Sync + Send + Bounded>(
        &self,
        data: &[T],
        number_of_rays: u32,
        velocity: f64,
        sample_rate: f64,
        scaling_factor: f64,
        do_snapshot_method: bool,
        progress_counter: &Arc<AtomicU32>,
    ) -> Vec<T> {
        let buffers: Vec<Vec<f64>> = data
            .iter()
            .enumerate()
            .map(|(idx, val)| (idx, *val))
            .collect::<Vec<(usize, T)>>()
            .par_chunks(1000)
            // .chunks(1000)
            .map(|chunk| {
                let result = self.simulate_for_chunk(
                    data.len(),
                    chunk,
                    number_of_rays,
                    velocity,
                    sample_rate,
                    scaling_factor,
                    do_snapshot_method,
                );
                {
                    let cloned_counter = Arc::clone(progress_counter);
                    cloned_counter.fetch_add(1, Ordering::AcqRel);
                }
                result
            })
            .collect();
        let max_len = buffers.iter().max_by_key(|vec| vec.len()).unwrap().len();
        let mut buffer = vec![0f64; max_len];
        for buffer_to_add in &buffers {
            buffer
                .iter_mut()
                .zip(buffer_to_add)
                .for_each(|(val, to_add)| *val += *to_add);
        }
        let mut had_to_clip = false;
        buffer
            .iter()
            .map(|val| {
                // clipping in case we exceed T's range
                // shouldn't be necessary if scaling_factor does its job
                num::cast::<f64, T>(*val).unwrap_or_else(|| {
                    if !had_to_clip {
                        had_to_clip = true;
                        println!("WARNING: Part of the resulting audio had to be clipped because it exceeded the file format's range. Please try a bigger scaling factor.");
                    }
                    if *val > 0f64 {
                        T::max_value()
                    } else {
                        T::min_value()
                    }
                })
            })
            .collect()
    }

    /// Internal logic for `simulate_for_time_span_internal`
    #[allow(clippy::too_many_arguments)]
    fn simulate_for_chunk<T: Num + NumCast + Clone + Copy + Sync + Send>(
        &self,
        data_len: usize,
        chunk: &[(usize, T)],
        number_of_rays: u32,
        velocity: f64,
        sample_rate: f64,
        scaling_factor: f64,
        do_snapshot_method: bool,
    ) -> Vec<f64> {
        let mut buffer: Vec<f64> = vec![0f64; data_len];
        for (idx, value) in chunk {
            let impulse_response = self.simulate_at_time(
                *idx as u32,
                number_of_rays,
                velocity,
                sample_rate,
                do_snapshot_method,
            );
            let buffer_to_add =
                impulse_response::apply_to_sample(&impulse_response, *value, *idx, scaling_factor);
            if buffer.len() < buffer_to_add.len() {
                buffer.resize(buffer_to_add.len(), 0f64);
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
    /// If `do_snapshot_method` is true, a static version of the scene at `time` is taken and simulation is run through that instead.
    pub fn simulate_at_time(
        &self,
        time: u32,
        number_of_rays: u32,
        velocity: f64,
        sample_rate: f64,
        do_snapshot_method: bool,
    ) -> Vec<f64> {
        let mut scene_data = self;
        let interp_scene_data;
        if do_snapshot_method {
            let interp_scene = self.scene.at_time(time);
            let chunks = interp_scene.chunks::<C>();
            interp_scene_data = Self {
                scene: interp_scene,
                chunks,
                maximum_bounds: self.maximum_bounds,
            };
            scene_data = &interp_scene_data;
        }

        let rt_results: Vec<(f64, u32)> = (0..number_of_rays)
            .flat_map(|_| scene_data.launch_ray(time, velocity, sample_rate))
            .collect();
        to_impulse_response(&rt_results, number_of_rays)
    }

    /// Launch a single ray into this `Scene`, and return its result.
    /// The direction it is launched in is a random position in the unit cube,
    /// which gets normalised in the ray's launch function.
    fn launch_ray(&self, time: u32, velocity: f64, sample_rate: f64) -> Vec<(f64, u32)> {
        let Emitter::Interpolated(emitter_coords, _, emission_type) =
            self.scene.emitter.at_time(time)
        else {
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
