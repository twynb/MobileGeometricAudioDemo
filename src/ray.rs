use std::ops::Mul;

use approx::abs_diff_eq;
use generic_array::ArrayLength;
use nalgebra::{base::Unit, Vector3};
use num::{Num, NumCast};
use rand::random;
use typenum::Unsigned;

use crate::{
    interpolation::Interpolation,
    intersection,
    scene::{SceneData, Surface},
    DEFAULT_SAMPLE_RATE,
};

/// The normal speed of sound in air at 20 °C, in m/s.
pub const DEFAULT_PROPAGATION_SPEED: f32 = 343.2;
/// The threshold below which rays get discarded.
const ENERGY_THRESHOLD: f32 = 0.00005;

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
    Found(bool, usize, u32, Vector3<f32>),
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
    /// The last surface this ray has intersected with, to avoid repeatedly bouncing off the same surface.
    last_intersected_surface: Option<usize>,
}

impl Ray {
    /// Create a new ray with the given parameters.
    /// This function is only relevant for testing purposes and shouldn't be used otherwise.
    pub const fn new(
        direction: Unit<Vector3<f32>>,
        origin: Vector3<f32>,
        energy: f32,
        time: u32,
        velocity: f32,
    ) -> Self {
        Self {
            direction,
            origin,
            energy,
            time,
            velocity,
            last_intersected_surface: None,
        }
    }

    /// Get the coordinates this ray is at at the given time.
    ///
    /// # Panics
    ///
    /// * If u32 cannot be cast to T, or T cannot be cast to f32
    pub fn coords_at_time<T: Num + NumCast>(&self, time: T) -> Vector3<f32> {
        let factor: f32 = num::cast(time - num::cast(self.time).unwrap()).unwrap();
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
        direction: Vector3<f32>,
        origin: Vector3<f32>,
        start_time: u32,
        velocity: f32,
        sample_rate: f32,
        scene_data: &SceneData<C>,
    ) -> Vec<(f32, u32)>
    where
        C: Unsigned + Mul<C>,
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
    ///
    /// KNOWN ISSUE: We lose some rays here (<1% in the extreme case of working with fully diffusing surfaces)
    /// because of floating point imprecisions, especially when they get into corners.
    /// This will be ignored for now because it's an edge case that will not lose us a significant amount of rays.
    fn bounce<C>(&mut self, scene_data: &SceneData<C>) -> Vec<(f32, u32)>
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
                None => self.energy = -1f32, // cancel the loop, we're out of bounds
                Some((is_receiver, index, time, coords)) => {
                    if is_receiver {
                        // do not change direction because we pass through receivers
                        result.push((self.energy, time));
                        allow_receiver = false;
                        self.origin = coords;
                        self.last_intersected_surface = None;
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
        time: u32,
        coords: Vector3<f32>,
        index: usize,
    ) where
        C: Unsigned + Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        let surface = scene_data.scene.surfaces[index].at_time(time);
        let Surface::Interpolated(_surface_coords, _time, material) = surface else {
            panic!("at_time() somehow returned a non-interpolated surface. This shouldn't happen.")
        };

        let mut normal = surface.normal();
        let mut new_direction = self.direction.into_inner();
        // we don't know which direction the normal is facing from the surface
        // but need it to be towards the ray's origin
        // so if the previous direction and the normal are in the same general direction,
        // we need to invert the normal to go in the correct direction
        if normal.dot(&new_direction) > 0f32 {
            normal *= -1f32;
        }
        if material.is_bounce_diffuse() {
            // new_direction doesn't have to be a unit vector yet, we'll normalise it later
            new_direction = Vector3::new(
                random::<f32>().mul_add(2f32, -1f32),
                random::<f32>().mul_add(2f32, -1f32),
                random::<f32>().mul_add(2f32, -1f32),
            );
            // new_direction needs to be in a hemisphere on the surface (can't bounce behind the surface)
            // => if dot product with normal is negative, it's going behind the surface
            // we also don't want a perfectly orthogonal bounce, so do <= rather than =
            while new_direction.dot(&normal) <= 0f32 {
                new_direction = Vector3::new(
                    random::<f32>().mul_add(2f32, -1f32),
                    random::<f32>().mul_add(2f32, -1f32),
                    random::<f32>().mul_add(2f32, -1f32),
                );
            }
        } else {
            new_direction -= 2f32 * (self.direction.dot(&normal)) * normal;
        }

        self.time = time;
        self.origin = coords;
        self.direction = Unit::new_normalize(new_direction);
        self.energy *= material.absorption_coefficient;
        self.last_intersected_surface = Some(index);
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
    ) -> Option<(bool, usize, u32, Vector3<f32>)>
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

        if dimension.position > dimension.bound {
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
        let (receivers, surfaces) = scene_data
            .chunks
            .objects_at_key_and_time(key, time_entry, time_exit);

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
            if let Some(last_index) = self.last_intersected_surface {
                if last_index == *surface_index {
                    // skip the last surface we bounced off of
                    continue;
                }
            }
            let Some((time, coords)) = intersection::intersect_ray_and_surface(
                self,
                &scene_data.scene.surfaces[*surface_index],
                time_entry,
                time_exit,
            ) else {
                // skip surfaces we don't intersect with
                continue;
            };
            if match result {
                IntersectionCheckResult::Found(_is_recv, _index, result_time, _coords) => {
                    time > result_time
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
            last_time: self.time,
            x: init_chunk_traversal_data_dimension(
                self.direction[0], // we can directly use direction as direction cosine because it's a unit vector
                C::to_i32() * C::to_i32(),
                self.origin.x,
                scene_data.chunks.size_x,
                scene_data
                    .chunks
                    .size_x
                    .mul_add(chunk_indices.0 as f32, scene_data.chunks.chunk_starts.x),
                self.time,
                self.velocity,
                C::to_u32(),
                chunk_indices.0,
            ),
            y: init_chunk_traversal_data_dimension(
                self.direction[1], // we can directly use direction as direction cosine because it's a unit vector
                C::to_i32(),
                self.origin.y,
                scene_data.chunks.size_y,
                scene_data
                    .chunks
                    .size_y
                    .mul_add(chunk_indices.1 as f32, scene_data.chunks.chunk_starts.y),
                self.time,
                self.velocity,
                C::to_u32(),
                chunk_indices.1,
            ),
            z: init_chunk_traversal_data_dimension(
                self.direction[2], // we can directly use direction as direction cosine because it's a unit vector
                1,
                self.origin.z,
                scene_data.chunks.size_z,
                scene_data
                    .chunks
                    .size_z
                    .mul_add(chunk_indices.2 as f32, scene_data.chunks.chunk_starts.z),
                self.time,
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
    direction_cosine: f32,
    key_increment: i32,
    origin_position: f32,
    chunk_width: f32,
    chunk_start: f32,
    start_time: u32,
    velocity: f32,
    num_chunks: u32,
    chunk_index: u32,
) -> ChunkTraversalDataDimension {
    if abs_diff_eq!(direction_cosine, 0f32) {
        ChunkTraversalDataDimension {
            position: f32::MAX,
            delta_position: 0f32,
            key_increment,
            time: 0f32,
            delta_time: 0f32,
            bound: 0f32,
        }
    } else if direction_cosine > 0f32 {
        let delta_direction = chunk_width / direction_cosine;
        let delta_time: f32 = delta_direction / velocity;
        ChunkTraversalDataDimension {
            position: (chunk_start + chunk_width - origin_position) / chunk_width * delta_direction,
            delta_position: delta_direction,
            key_increment,
            time: ((chunk_start + chunk_width - origin_position) / chunk_width)
                .mul_add(delta_time, start_time as f32),
            delta_time,
            bound: (num_chunks - chunk_index) as f32 * delta_direction,
        }
    } else {
        let delta_direction = -chunk_width / direction_cosine;
        let delta_time: f32 = delta_direction / velocity;
        ChunkTraversalDataDimension {
            position: (origin_position - chunk_start) / chunk_width * delta_direction,
            delta_position: delta_direction,
            key_increment: -key_increment,
            time: ((origin_position - chunk_start) / chunk_width)
                .mul_add(delta_time, start_time as f32),
            delta_time,
            bound: (chunk_index + 1) as f32 * delta_direction,
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
            last_intersected_surface: None,
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
    position: f32,
    delta_position: f32,
    key_increment: i32,
    // store time as a float here to avoid rounding errors
    time: f32,
    delta_time: f32,
    bound: f32,
}
