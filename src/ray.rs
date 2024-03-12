use std::ops::Mul;

use approx::abs_diff_eq;
use generic_array::ArrayLength;
use nalgebra::{base::Unit, Vector3};
use num::{Num, NumCast};
use typenum::Unsigned;

use crate::{
    bounce::{bounce_off_surface_with_normal, random_direction_in_hemisphere},
    interpolation::Interpolation,
    intersection,
    scene::{SceneData, Surface},
    DEFAULT_SAMPLE_RATE,
};

/// The normal speed of sound in air at 20 °C, in m/s.
pub const DEFAULT_PROPAGATION_SPEED: f64 = 343.2;
/// The threshold below which rays get discarded.
const ENERGY_THRESHOLD: f64 = 0.000001;

/// The result after checking for an intersection.
/// * `Found`: found an intersecting surface.
/// * `NoIntersection`: No intersection, continue propagating this ray.
/// * `OutOfBounds`: The ray has exited the scene, no need to propagate further.
#[derive(Debug, Clone, Copy, PartialEq)]
enum IntersectionCheckResult {
    /// An intersection has been found.
    /// Variables represent:
    /// * Whether the intersection is with a receiver as opposed to a surface
    /// * The surface's index (or 0 for receivers)
    /// * The intersection time
    /// * The intersection position's coordinates.
    Found(bool, usize, f64, Vector3<f64>),
    /// No intersection has been found, continue propagating this ray.
    NoIntersection,
    /// The ray has gone out of bounds. No need to bother propagating it further.
    OutOfBounds,
}

impl IntersectionCheckResult {
    /// Check whether this `IntersectionCheckResult` is of type "Found".
    const fn is_found(&self) -> bool {
        matches!(self, Self::Found(_is_recv, _index, _time, _coords))
    }
}

#[derive(Clone, PartialEq, Debug, Copy)]
/// A ray to bounce through the scene.
pub struct Ray {
    /// The direction to shoot the ray in.
    pub direction: Unit<Vector3<f64>>,
    /// The origin position to shoot the ray from.
    pub origin: Vector3<f64>,
    /// The ray's current energy - this should get decremented
    /// with every bounce.
    /// This starts out at 1.0f64 and if it goes near/below 0f64, this ray can
    /// be discarded.
    pub energy: f64,
    /// The time at which the ray is launched, in samples. - this
    /// should get incremented with every bounce.
    pub time: f64,
    /// The velocity at which the ray moves, in meters per sample.
    /// This should usually be ``crate::ray::DEFAULT_PROPAGATION_SPEED`` / ``crate::DEFAULT_SAMPLE_RATE``.
    pub velocity: f64,
}

impl Ray {
    /// Create a new ray with the given parameters.
    /// This function is only relevant for testing purposes and shouldn't be used otherwise.
    pub fn new(
        direction: Unit<Vector3<f64>>,
        origin: Vector3<f64>,
        energy: f64,
        time: u32,
        velocity: f64,
    ) -> Self {
        Self {
            direction,
            origin,
            energy,
            time: <f64 as From<u32>>::from(time),
            velocity,
        }
    }

    /// Get the coordinates this ray is at at the given time.
    ///
    /// # Panics
    ///
    /// * If u32 cannot be cast to T, or T cannot be cast to f64
    pub fn coords_at_time<T: Num + NumCast>(&self, time: T) -> Vector3<f64> {
        let factor: f64 = num::cast(time - num::cast(self.time).unwrap()).unwrap();
        let direction = self.direction.into_inner() * factor * self.velocity;
        self.origin + direction
    }

    /// Launch a ray from the given origin in the given direction. Returns
    /// both the energy and time at which the ray hits the listener, or None
    /// if it doesn't.
    ///
    /// # Arguments
    ///
    /// * `direction`: The direction to launch the ray in. This will be normalised, so it doesn't have to be normalised before.
    /// * `origin`: The origin coordinates to launch the ray from.
    /// * `start_time`: The time at which the ray is launched.
    /// * `velocity`: The ray's velocity, in meters per second.
    /// * `sample_rate`: The sample rate at which the simulation is run.
    /// * `scene`: The scene to bounce in.
    /// * `chunks`: The chunks for the scene.
    /// * `maximum_bounds`: The scene's outer bounds.
    pub fn launch<C>(
        direction: Vector3<f64>,
        origin: Vector3<f64>,
        start_time: u32,
        velocity: f64,
        sample_rate: f64,
        scene_data: &SceneData<C>,
    ) -> Vec<(f64, u32)>
    where
        C: Unsigned + Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        let mut ray = Self {
            direction: Unit::new_normalize(direction),
            origin,
            velocity: velocity / sample_rate,
            time: <f64 as From<u32>>::from(start_time),
            ..Default::default()
        };

        ray.bounce(scene_data)
    }

    /// Bounce this ray through the given scene.
    ///
    /// KNOWN ISSUE: We lose some rays here (<1% in the extreme case of working with fully diffusing surfaces)
    /// because of floating point imprecisions, especially when they get into corners.
    /// This will be ignored for now because it's an edge case that will not lose us a significant amount of rays.
    fn bounce<C>(&mut self, scene_data: &SceneData<C>) -> Vec<(f64, u32)>
    where
        C: Unsigned + Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        let mut allow_receiver = true;
        let mut result = vec![];
        while self.energy > ENERGY_THRESHOLD {
            let mut chunk_traversal_data = self.init_chunk_traversal_data(scene_data);
            match self.traverse(scene_data, &mut chunk_traversal_data, allow_receiver) {
                None => {
                    self.energy = -1f64; // cancel the loop, we're out of bounds
                }
                Some((is_receiver, index, time, coords)) => {
                    if is_receiver {
                        // do not change direction because we pass through receivers
                        result.push((self.energy, time.round() as u32));
                        allow_receiver = false;
                    } else {
                        allow_receiver = true;
                        self.bounce_from_intersection(scene_data, time, coords, index);
                    }
                }
            }
        }
        result
    }

    /// Bounce off of an intersection with a surface with the given index.
    /// The surface material is used to determine how much energy the ray loses
    /// and whether it's reflected specularly or refracted.
    /// for refraction, get a random vector within the hemisphere on top of the surface
    /// and make that the new normal vector.
    /// for specular reflection, calculate the bouncing angle.
    fn bounce_from_intersection<C>(
        &mut self,
        scene_data: &SceneData<C>,
        time: f64,
        coords: Vector3<f64>,
        index: usize,
    ) where
        C: Unsigned + Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        let looped_time = scene_data
            .scene
            .loop_duration
            .map_or(time.round() as u32, |duration| {
                time.round() as u32 % duration
            });
        let surface = scene_data.scene.surfaces[index].at_time(looped_time);
        let Surface::Interpolated(_surface_coords, _time, surface_data) = surface else {
            panic!("at_time() somehow returned a non-interpolated surface. This shouldn't happen.")
        };
        let material = surface_data.material;

        let normal = surface.normal();

        let new_direction = if material.is_bounce_diffuse() {
            random_direction_in_hemisphere(&normal)
        } else {
            bounce_off_surface_with_normal(self.direction.into_inner(), &normal)
        };

        self.time = time;
        self.origin = coords;
        self.direction = Unit::new_normalize(new_direction);
        self.energy *= material.absorption_coefficient;
    }

    /// Traverse through a scene chunk by chunk.
    /// This is based on [Cleary/Wyvill's paper from 1988](https://link.springer.com/article/10.1007/BF01905559)
    ///
    /// `chunk_traversal_data` holds the information on where the ray
    /// currently is, and is updated in a loop until either a chunk
    /// with an intersection is found or the ray exits the scene.
    fn traverse<C>(
        &self,
        scene_data: &SceneData<C>,
        chunk_traversal_data: &mut ChunkTraversalData,
        allow_receiver: bool,
    ) -> Option<(bool, usize, f64, Vector3<f64>)>
    where
        C: Unsigned + Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        loop {
            if chunk_traversal_data.x.position <= chunk_traversal_data.y.position
                && chunk_traversal_data.x.position <= chunk_traversal_data.z.position
            {
                match self.traverse_to_next_chunk(
                    &mut chunk_traversal_data.key,
                    &mut chunk_traversal_data.last_time,
                    &mut chunk_traversal_data.x,
                    scene_data,
                    allow_receiver,
                ) {
                    IntersectionCheckResult::Found(is_receiver, index, time, coords) => {
                        return Some((is_receiver, index, time, coords))
                    }
                    IntersectionCheckResult::OutOfBounds => return None,
                    IntersectionCheckResult::NoIntersection => (), // continue if no intersection
                };
            } else if chunk_traversal_data.y.position <= chunk_traversal_data.x.position
                && chunk_traversal_data.y.position <= chunk_traversal_data.z.position
            {
                match self.traverse_to_next_chunk(
                    &mut chunk_traversal_data.key,
                    &mut chunk_traversal_data.last_time,
                    &mut chunk_traversal_data.y,
                    scene_data,
                    allow_receiver,
                ) {
                    IntersectionCheckResult::Found(is_receiver, index, time, coords) => {
                        return Some((is_receiver, index, time, coords))
                    }
                    IntersectionCheckResult::OutOfBounds => return None,
                    IntersectionCheckResult::NoIntersection => (), // continue if no intersection
                };
            } else {
                match self.traverse_to_next_chunk(
                    &mut chunk_traversal_data.key,
                    &mut chunk_traversal_data.last_time,
                    &mut chunk_traversal_data.z,
                    scene_data,
                    allow_receiver,
                ) {
                    IntersectionCheckResult::Found(is_receiver, index, time, coords) => {
                        return Some((is_receiver, index, time, coords))
                    }
                    IntersectionCheckResult::OutOfBounds => return None,
                    IntersectionCheckResult::NoIntersection => (), // continue if no intersection
                };
            }
        }
    }

    /// Check for an intersection in the current chunk,
    /// then traverse to the next chunk.
    /// If an intersection is found in the current chunk, return that.
    /// If the next chunk would be outside the scene bounds, return accordingly.
    /// Otherwise, continue.
    fn traverse_to_next_chunk<C>(
        &self,
        key: &mut i32,
        last_time: &mut u32,
        dimension: &mut ChunkTraversalDataDimension,
        scene_data: &SceneData<C>,
        allow_receiver: bool,
    ) -> IntersectionCheckResult
    where
        C: Unsigned + Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        let intersection = self.intersection_check_in_chunk(
            *key as u32,
            *last_time,
            dimension.time.ceil() as u32,
            scene_data,
            allow_receiver,
        );
        if intersection.is_found() {
            return intersection;
        }

        *last_time = dimension.time.trunc() as u32;
        *key += dimension.key_increment;
        dimension.position += dimension.delta_position;
        dimension.time += dimension.delta_time;

        if dimension.position >= dimension.bound {
            return IntersectionCheckResult::OutOfBounds;
        }

        IntersectionCheckResult::NoIntersection
    }

    /// Check whether there are any intersections in the current chunk.
    /// If the chunk does not contain anything, return out early.
    fn intersection_check_in_chunk<C>(
        &self,
        key: u32,
        time_entry: u32,
        time_exit: u32,
        scene_data: &SceneData<C>,
        allow_receiver: bool,
    ) -> IntersectionCheckResult
    where
        C: Unsigned + Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        if !scene_data.chunks.is_chunk_set(key as usize) {
            return IntersectionCheckResult::NoIntersection;
        }
        let (receivers, surfaces) = scene_data.chunks.objects_at_key_and_time(
            key,
            time_entry,
            time_exit,
            scene_data.scene.loop_duration,
        );

        let result = if allow_receiver {
            self.intersection_check_receiver_in_chunk(&receivers, scene_data, time_entry, time_exit)
        } else {
            IntersectionCheckResult::NoIntersection
        };

        self.intersection_check_surface_in_chunk(
            &surfaces, scene_data, time_entry, time_exit, result,
        )
    }

    /// Check if this ray intersects with the receiver inside this chunk.
    /// If there is no receiver inside this chunk, skip the check.
    fn intersection_check_receiver_in_chunk<C>(
        &self,
        receivers: &[usize],
        scene_data: &SceneData<C>,
        time_entry: u32,
        time_exit: u32,
    ) -> IntersectionCheckResult
    where
        C: Unsigned + Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        if receivers.is_empty() {
            return IntersectionCheckResult::NoIntersection;
        }
        // as of current we only have one receiver - this logic might change in the future
        if let Some((time, coords)) = intersection::intersect_ray_and_receiver(
            self,
            &scene_data.scene.receiver,
            time_entry,
            time_exit,
            scene_data.scene.loop_duration,
        ) {
            return IntersectionCheckResult::Found(true, 0, time, coords);
        }
        IntersectionCheckResult::NoIntersection
    }

    /// Check if this ray intersects with surfaces inside this chunk.
    /// Surfaces that the ray has last intersected with are skipped.
    ///
    /// For surfaces the ray does intersect with, if the intersection
    /// is earlier than previously found intersections (including the one from `result`),
    /// replace `result` with it and eventually return the earliest intersection.
    fn intersection_check_surface_in_chunk<C>(
        &self,
        surfaces: &[usize],
        scene_data: &SceneData<C>,
        time_entry: u32,
        time_exit: u32,
        mut result: IntersectionCheckResult,
    ) -> IntersectionCheckResult
    where
        C: Unsigned + Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        for surface_index in surfaces {
            let Some((time, coords)) = intersection::intersect_ray_and_surface(
                self,
                &scene_data.scene.surfaces[*surface_index],
                time_entry,
                time_exit,
                scene_data.scene.loop_duration,
            ) else {
                // skip surfaces we don't intersect with
                continue;
            };

            if match result {
                IntersectionCheckResult::Found(_is_recv, _index, result_time, _coords) => {
                    time < result_time
                }
                _ => true,
            } {
                result = IntersectionCheckResult::Found(false, *surface_index, time, coords);
            }
        }

        result
    }

    /// Initialise the chunk traversal data.
    /// We first calculate the key of the chunk the ray starts in,
    /// then initialise the `ChunkTraversalData` with that and the individual dimensions.
    fn init_chunk_traversal_data<C>(&self, scene_data: &SceneData<C>) -> ChunkTraversalData
    where
        C: Unsigned + Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        let chunk_indices = scene_data.chunks.coords_to_chunk_index(&self.origin);
        let key = scene_data
            .chunks
            .key_for_index(chunk_indices.0, chunk_indices.1, chunk_indices.2)
            as i32;
        ChunkTraversalData {
            key,
            last_time: self.time.floor() as u32,
            x: init_chunk_traversal_data_dimension(
                self.direction[0], // we can directly use direction as direction cosine because it's a unit vector
                C::to_i32() * C::to_i32(),
                self.origin.x,
                scene_data.chunks.size_x,
                scene_data.chunks.size_x.mul_add(
                    <f64 as From<u32>>::from(chunk_indices.0),
                    scene_data.chunks.chunk_starts.x,
                ),
                self.time.floor() as u32,
                self.velocity,
                C::to_u32(),
                chunk_indices.0,
            ),
            y: init_chunk_traversal_data_dimension(
                self.direction[1], // we can directly use direction as direction cosine because it's a unit vector
                C::to_i32(),
                self.origin.y,
                scene_data.chunks.size_y,
                scene_data.chunks.size_y.mul_add(
                    <f64 as From<u32>>::from(chunk_indices.1),
                    scene_data.chunks.chunk_starts.y,
                ),
                self.time.floor() as u32,
                self.velocity,
                C::to_u32(),
                chunk_indices.1,
            ),
            z: init_chunk_traversal_data_dimension(
                self.direction[2], // we can directly use direction as direction cosine because it's a unit vector
                1,
                self.origin.z,
                scene_data.chunks.size_z,
                scene_data.chunks.size_z.mul_add(
                    <f64 as From<u32>>::from(chunk_indices.2),
                    scene_data.chunks.chunk_starts.z,
                ),
                self.time.floor() as u32,
                self.velocity,
                C::to_u32(),
                chunk_indices.2,
            ),
        }
    }
}

/// Initialise the chunk traversal data for a single dimension.
#[allow(clippy::too_many_arguments)]
fn init_chunk_traversal_data_dimension(
    direction_cosine: f64,
    key_increment: i32,
    origin_position: f64,
    chunk_width: f64,
    chunk_start: f64,
    start_time: u32,
    velocity: f64,
    num_chunks: u32,
    chunk_index: u32,
) -> ChunkTraversalDataDimension {
    if abs_diff_eq!(direction_cosine, 0f64) {
        ChunkTraversalDataDimension {
            position: f64::MAX,
            delta_position: 0f64,
            key_increment,
            time: 0f64,
            delta_time: 0f64,
            bound: 0f64,
        }
    } else if direction_cosine > 0f64 {
        let delta_position = chunk_width / direction_cosine;
        let delta_time: f64 = delta_position / velocity;
        let position = (chunk_start + chunk_width - origin_position) / chunk_width * delta_position;
        let bound = if position > delta_position / 2f64 {
            // if we're in the second half of the chunk, we'd rather the border be slightly into the OOB chunk
            // to reduce the risk of rounding errors when we're near the end of the chunk each time
            (<f64 as From<u32>>::from(num_chunks - chunk_index) + 0.1) * delta_position
        } else {
            // if we're in the first half of the chunk, we'd rather the border be slightly into the last chunk
            // to reduce the risk of rounding errors when we're near the beginning of the chunk each time
            (<f64 as From<u32>>::from(num_chunks - chunk_index) - 0.1f64) * delta_position
        };
        ChunkTraversalDataDimension {
            position,
            delta_position,
            key_increment,
            time: ((chunk_start + chunk_width - origin_position) / chunk_width)
                .mul_add(delta_time, <f64 as From<u32>>::from(start_time)),
            delta_time,
            bound,
        }
    } else {
        let delta_position = -chunk_width / direction_cosine;
        let delta_time: f64 = delta_position / velocity;
        let position = (origin_position - chunk_start) / chunk_width * delta_position;
        let bound = if position > delta_position / 2f64 {
            // if we're in the second half of the chunk, we'd rather the border be slightly into the OOB chunk
            // to reduce the risk of rounding errors when we're near the end of the chunk each time
            (<f64 as From<u32>>::from(chunk_index) + 1.1) * delta_position
        } else {
            // if we're in the first half of the chunk, we'd rather the border be slightly into the last chunk
            // to reduce the risk of rounding errors when we're near the beginning of the chunk each time
            (<f64 as From<u32>>::from(chunk_index) + 0.9f64) * delta_position
        };
        ChunkTraversalDataDimension {
            position,
            delta_position,
            key_increment: -key_increment,
            time: ((origin_position - chunk_start) / chunk_width)
                .mul_add(delta_time, <f64 as From<u32>>::from(start_time)),
            delta_time,
            // truncate bound because it doesn't need to be that specific, & float rounding issues in the bound can lead to OOB issues
            // we'd rather have it be a bit too small (=> nothing changes) than a bit too large (=> we don't return out where we should)
            bound,
        }
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            direction: Unit::new_normalize(Vector3::new(0f64, 1f64, 0f64)),
            origin: Vector3::new(0f64, 0f64, 0f64),
            energy: 1f64,
            time: 0f64,
            velocity: DEFAULT_PROPAGATION_SPEED / DEFAULT_SAMPLE_RATE,
        }
    }
}

/// Data required for chunk traversal as per CW88
#[derive(Clone, Copy, Debug, PartialEq)]
struct ChunkTraversalData {
    key: i32,
    last_time: u32,
    x: ChunkTraversalDataDimension,
    y: ChunkTraversalDataDimension,
    z: ChunkTraversalDataDimension,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ChunkTraversalDataDimension {
    position: f64,
    delta_position: f64,
    key_increment: i32,
    // store time as a float here to avoid rounding errors
    time: f64,
    delta_time: f64,
    bound: f64,
}
