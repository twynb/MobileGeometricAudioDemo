use std::ops::Mul;

use generic_array::ArrayLength;
use nalgebra::{base::Unit, Vector3};
use num::{Num, NumCast};
use typenum::Unsigned;

use crate::{intersection, SceneData, DEFAULT_SAMPLE_RATE};

/// The normal speed of sound in air at 20 °C, in m/s.
pub const DEFAULT_PROPAGATION_SPEED: f32 = 343.2;

/// The result after checking for an intersection.
/// * `Found`: found an intersecting surface.
/// * `NoIntersection`: No intersection, continue propagating this ray.
/// * `OutOfBounds`: The ray has exited the scene, no need to propagate further.
enum IntersectionCheckResult {
    /// An intersection has been found.
    /// Variables represent:
    /// * Whether the intersection is with a receiver as opposed to a surface
    /// * The surface's index (or 0 for receivers)
    /// * The intersection time
    /// * The intersection position's coordinates.
    Found(bool, usize, u32, Vector3<f32>),
    /// No intersection has been found, continue propagating this ray.
    NoIntersection,
    /// The ray has gone out of bounds. No need to bother propagating it further.
    OutOfBounds,
}

impl IntersectionCheckResult {
    /// Check whether this `IntersectionCheckResult` is of type "Found".
    const fn is_found(&self) -> bool {
        matches!(self, IntersectionCheckResult::Found(_is_recv, _index, _time, _coords))
    }
}

#[derive(Clone, PartialEq)]
/// A ray to bounce through the scene.
pub struct Ray {
    /// The direction to shoot the ray in.
    pub direction: Unit<Vector3<f32>>,
    /// The origin position to shoot the ray from.
    pub origin: Vector3<f32>,
    /// The ray's current energy - this should get decremented
    /// with every bounce.
    /// This starts out at 1.0f32 and if it goes near/below 0f32, this ray can
    /// be discarded.
    pub energy: f32,
    /// The time at which the ray arrives at the receiver, in samples. - this
    /// should get incremented with every bounce.
    pub time: u32,
    /// The velocity at which the ray moves, in meters per sample.
    /// This should usually be ``crate::ray::DEFAULT_PROPAGATION_SPEED`` / ``crate::DEFAULT_SAMPLE_RATE``.
    pub velocity: f32,
}

impl Ray {
    /// Get the coordinates this ray is at at the given time.
    /// 
    /// # Panics
    /// 
    /// * If u32 cannot be cast to T, or T cannot be cast to f32
    pub fn coords_at_time<T: Num + NumCast>(&self, time: T) -> Vector3<f32> {
        let factor: f32 = num::cast(time - num::cast(self.time).unwrap()).unwrap();
        let direction = self.direction.into_inner() * factor;
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
    pub fn launch<C: Unsigned>(
        direction: Vector3<f32>,
        origin: Vector3<f32>,
        start_time: u32,
        velocity: f32,
        sample_rate: f32,
        scene_data: &SceneData<C>,
    ) -> Option<(f32, u32)>
    where
        C: Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        let mut ray = Self {
            direction: Unit::new_normalize(direction),
            origin,
            velocity: velocity / sample_rate,
            time: start_time,
            ..Default::default()
        };

        ray.bounce(scene_data)
    }

    /// Bounce this ray through the given scene.
    fn bounce<C: Unsigned>(&mut self, scene_data: &SceneData<C>) -> Option<(f32, u32)>
    where
        C: Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        let mut chunk_traversal_data = self.init_chunk_traversal_data(scene_data);
        match self.traverse(scene_data, &mut chunk_traversal_data) {
            None => None,
            Some((is_receiver, index, time, coords)) => {
                if is_receiver {
                    return Some((self.energy, time));
                }
                todo!("TODO");
            }
        }
    }

    /// Traverse through a scene chunk by chunk.
    /// This is based on [Cleary/Wyvill's paper from 1988](https://link.springer.com/article/10.1007/BF01905559)
    ///
    /// `chunk_traversal_data` holds the information on where the ray
    /// currently is, and is updated in a loop until either a chunk
    /// with an intersection is found or the ray exits the scene.
    fn traverse<C: Unsigned>(
        &self,
        scene_data: &SceneData<C>,
        chunk_traversal_data: &mut ChunkTraversalData,
    ) -> Option<(bool, usize, u32, Vector3<f32>)>
    where
        C: Mul<C>,
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

    ///
    fn traverse_to_next_chunk<C: Unsigned>(
        &self,
        key: &mut i32,
        last_time: &mut u32,
        dimension: &mut ChunkTraversalDataDimension,
        scene_data: &SceneData<C>,
    ) -> IntersectionCheckResult
    where
        C: Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        let intersection = self.intersection_check_in_chunk(
            *key as u32,
            *last_time,
            dimension.time.ceil() as u32,
            scene_data,
        );
        if intersection.is_found() {
            return intersection;
        }

        if dimension.position >= dimension.bound {
            return IntersectionCheckResult::OutOfBounds;
        }

        *last_time = dimension.time.trunc() as u32;
        *key += dimension.key_increment;
        dimension.position += dimension.delta_position;
        dimension.time += dimension.delta_time;

        IntersectionCheckResult::NoIntersection
    }

    fn intersection_check_in_chunk<C: Unsigned>(
        &self,
        key: u32,
        time_entry: u32,
        time_exit: u32,
        scene_data: &SceneData<C>,
    ) -> IntersectionCheckResult
    where
        C: Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        if !scene_data.chunks.is_chunk_set(key as usize) {
            return IntersectionCheckResult::NoIntersection;
        }
        let (receivers, surfaces) = scene_data
            .chunks
            .objects_at_key_and_time(key, time_entry, time_exit);
        let mut result: IntersectionCheckResult = IntersectionCheckResult::NoIntersection;
        if !receivers.is_empty() {
            // as of current we only have one receiver - this logic might change in the future
            if let Some((time, coords)) = intersection::intersect_ray_and_receiver(
                self,
                &scene_data.scene.receiver,
                time_entry,
                time_exit,
            ) {
                result = IntersectionCheckResult::Found(true, 0, time, coords);
            }
        }

        for surface_index in surfaces {
            if let Some((time, coords)) = intersection::intersect_ray_and_surface(
                self,
                &scene_data.scene.surfaces[surface_index],
                time_entry,
                time_exit,
            ) {
                if match result {
                    IntersectionCheckResult::Found(_is_recv, _index, result_time, _coords) => {
                        time > result_time
                    }
                    _ => true,
                } {
                    result = IntersectionCheckResult::Found(false, surface_index, time, coords);
                }
            }
        }

        result
    }

    fn init_chunk_traversal_data<C: Unsigned>(
        &self,
        scene_data: &SceneData<C>,
    ) -> ChunkTraversalData
    where
        C: Mul<C>,
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
            last_time: self.time,
            x: init_chunk_traversal_data_dimension(
                self.direction[0], // we can directly use direction as direction cosine because it's a unit vector
                C::to_i32() * C::to_i32(),
                self.origin.x,
                scene_data.chunks.size_x,
                scene_data.chunks.chunk_starts.x + scene_data.chunks.size_x * chunk_indices.0 as f32,
                self.time,
                self.velocity,
                scene_data.maximum_bounds.0.x,
                scene_data.maximum_bounds.1.x,
            ),
            y: init_chunk_traversal_data_dimension(
                self.direction[1], // we can directly use direction as direction cosine because it's a unit vector
                C::to_i32(),
                self.origin.y,
                scene_data.chunks.size_y,
                scene_data.chunks.chunk_starts.y + scene_data.chunks.size_y * chunk_indices.1 as f32,
                self.time,
                self.velocity,
                scene_data.maximum_bounds.0.y,
                scene_data.maximum_bounds.1.y,
            ),
            z: init_chunk_traversal_data_dimension(
                self.direction[2], // we can directly use direction as direction cosine because it's a unit vector
                1,
                self.origin.z,
                scene_data.chunks.size_z,
                scene_data.chunks.chunk_starts.z + scene_data.chunks.size_z * chunk_indices.2 as f32,
                self.time,
                self.velocity,
                scene_data.maximum_bounds.0.z,
                scene_data.maximum_bounds.1.z,
            ),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn init_chunk_traversal_data_dimension(
    direction_cosine: f32,
    key_increment: i32,
    origin_position: f32,
    chunk_width: f32,
    chunk_start: f32,
    start_time: u32,
    velocity: f32,
    minimum_bound: f32,
    maximum_bound: f32,
) -> ChunkTraversalDataDimension {
    if direction_cosine <= 0.0001f32 {
        ChunkTraversalDataDimension {
            position: f32::MAX,
            delta_position: 0f32,
            key_increment,
            time: 0f32,
            delta_time: 0f32,
            bound: 0f32,
        }
    } else if direction_cosine > 0f32 {
        let delta_direction = 1f32 / direction_cosine;
        let delta_time: f32 = delta_direction * chunk_width / velocity;
        ChunkTraversalDataDimension {
            position: (chunk_start + chunk_width - origin_position) * delta_direction,
            delta_position: delta_direction,
            key_increment,
            time: start_time as f32
                + ((chunk_start + chunk_width - origin_position) * delta_time / chunk_width),
            delta_time,
            bound: maximum_bound,
        }
    } else {
        let delta_direction = -1f32 / direction_cosine;
        let delta_time: f32 = delta_direction * chunk_width / velocity;
        ChunkTraversalDataDimension {
            position: (origin_position - chunk_start) * delta_direction,
            delta_position: delta_direction,
            key_increment: -key_increment,
            time: start_time as f32 + ((origin_position - chunk_start) * delta_time / chunk_width),
            delta_time,
            bound: minimum_bound,
        }
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            direction: Unit::new_normalize(Vector3::new(0f32, 1f32, 0f32)),
            origin: Vector3::new(0f32, 0f32, 0f32),
            energy: 1f32,
            time: 0,
            velocity: DEFAULT_PROPAGATION_SPEED / DEFAULT_SAMPLE_RATE,
        }
    }
}

/// Data required for chunk traversal as per CW88
struct ChunkTraversalData {
    key: i32,
    last_time: u32,
    x: ChunkTraversalDataDimension,
    y: ChunkTraversalDataDimension,
    z: ChunkTraversalDataDimension,
}

struct ChunkTraversalDataDimension {
    position: f32,
    delta_position: f32,
    key_increment: i32,
    // store time as a float here to avoid rounding errors
    time: f32,
    delta_time: f32,
    bound: f32,
}
