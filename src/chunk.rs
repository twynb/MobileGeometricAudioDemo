use generic_array::{ArrayLength, GenericArray};
use itertools::Itertools;
use nalgebra::Vector3;
use num::integer::Average;
use std::collections::HashMap;
use std::ops::Mul;
use typenum::{operator_aliases::Cube, Unsigned};

use crate::{
    interpolation,
    scene::{CoordinateKeyframe, Receiver, Scene, Surface, SurfaceKeyframe},
    scene_bounds,
    scene_bounds::MaximumBounds,
    test_utils,
};

/// A single chunk entry. Chunk entries are either static
/// (i.e. they just hold an object index that stays in this chunk for
/// the entirety of the scene), dynamic (i.e. they also hold timestamps
/// for when the object enters/exits the chunk) or final (i.e. they only hold
/// a timestamp for when the object enters the chunk). The timestamp is inclusive,
/// meaning that at the last timestamp, the object still is within the chunk).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TimedChunkEntry {
    Dynamic(usize, u32, u32),
    Static(usize),
    Final(usize, u32),
}

impl TimedChunkEntry {
    const fn object_index(&self) -> usize {
        match self {
            Self::Dynamic(index, _entry, _exit) => *index,
            Self::Static(index) => *index,
            Self::Final(index, _entry) => *index,
        }
    }
}

/// A chunk within the scene. Chunks hold a vector of `TimedChunkEntry` entries for
/// surfaces and receivers that are inside the chunk at some point in the scene.
#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct SceneChunk {
    pub surfaces: Vec<TimedChunkEntry>,
    pub receivers: Vec<TimedChunkEntry>,
}

impl SceneChunk {
    /// Get the indices of all objects that are in this chunk at the given time.
    ///
    /// For the receivers (the first vector), the index doesn't mean anything as of current
    /// as there can only be one receiver.
    fn objects_at_time(
        &self,
        time_entry: u32,
        time_exit: u32,
        loop_duration: Option<u32>,
    ) -> (Vec<usize>, Vec<usize>) {
        let (loop_entry, loop_exit, time_entry, time_exit) =
            loop_duration.map_or((0, 0, time_entry, time_exit), |duration| {
                (
                    time_entry / duration,
                    time_exit / duration,
                    time_entry % duration,
                    time_exit % duration,
                )
            });
        if loop_entry == loop_exit {
            // everything takes place in the same loop iteration or we don't loop at all
            (
                self.receivers
                    .iter()
                    .filter_map(|entry| filter_map_entry_within_time(entry, time_entry, time_exit))
                    .unique()
                    .collect(),
                self.surfaces
                    .iter()
                    .filter_map(|entry| filter_map_entry_within_time(entry, time_entry, time_exit))
                    .unique()
                    .collect(),
            )
        } else if loop_exit - loop_entry >= 2 || time_exit >= time_entry {
            // if we run through the full loop all in one go, just return every object we have
            (
                self.receivers
                    .iter()
                    .map(TimedChunkEntry::object_index)
                    .unique()
                    .collect(),
                self.surfaces
                    .iter()
                    .map(TimedChunkEntry::object_index)
                    .unique()
                    .collect(),
            )
        } else {
            (
                self.receivers
                    .iter()
                    .filter_map(|entry| {
                        filter_map_entry_within_time_with_loop(entry, time_entry, time_exit)
                    })
                    .unique()
                    .collect(),
                self.surfaces
                    .iter()
                    .filter_map(|entry| {
                        filter_map_entry_within_time_with_loop(entry, time_entry, time_exit)
                    })
                    .unique()
                    .collect(),
            )
        }
    }
}

/// Filter whether the given entry is within the given
/// time frame, and return either its index or None
/// accordingly.
const fn filter_map_entry_within_time(
    entry: &TimedChunkEntry,
    time_entry: u32,
    time_exit: u32,
) -> Option<usize> {
    match entry {
        TimedChunkEntry::Static(index) => Some(*index),
        TimedChunkEntry::Final(index, entry) => {
            if *entry <= time_entry {
                Some(*index)
            } else {
                None
            }
        }
        TimedChunkEntry::Dynamic(index, entry, exit) => {
            if *entry <= time_exit && *exit >= time_entry {
                Some(*index)
            } else {
                None
            }
        }
    }
}

/// Filter whether the given entry is within the given
/// time frame, and return either its index or None
/// accordingly.
/// This is meant specifically for a case where `time_exit` takes place one loop after `time_entry`
/// and `time_exit` occurs earlier in the loop than `time_entry`.
/// In this case, we only need to check if one of these conditions applies:
/// a. the object has entered the chunk between 0 and `time_exit` (=> it shows up in the new loop)
/// b. the object hasn't exited the chunk between 0 and `time_entry` (=> it shows up in the old loop)
const fn filter_map_entry_within_time_with_loop(
    entry: &TimedChunkEntry,
    time_entry: u32,
    time_exit: u32,
) -> Option<usize> {
    match entry {
        TimedChunkEntry::Static(index) => Some(*index),
        TimedChunkEntry::Final(index, time_object_entry) => {
            if *time_object_entry <= time_entry {
                Some(*index)
            } else {
                None
            }
        }
        TimedChunkEntry::Dynamic(index, time_object_entry, time_object_exit) => {
            if *time_object_entry <= time_exit || *time_object_exit >= time_entry {
                Some(*index)
            } else {
                None
            }
        }
    }
}
impl PartialEq for SceneChunk {
    fn eq(&self, other: &Self) -> bool {
        test_utils::unordered_eq_without_duplicates(&self.surfaces, &other.surfaces)
            && test_utils::unordered_eq_without_duplicates(&self.receivers, &other.receivers)
    }
}

/// Data necessary to describe a scene as a set of chunks.
/// Keys for the `set_chunks` array as well as the `chunks` map
/// are calculated as (x << 16 + y << 8 + z), with x/y/z each being
/// an up to 8-bit index for the given chunk.
#[derive(Clone, Debug, PartialEq)]
pub struct Chunks<C>
where
    C: Unsigned + Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    /// An array with booleans indicating whether the given chunk has any data whatsoever.
    pub set_chunks: GenericArray<bool, Cube<C>>,
    /// The map of chunks holding actual data.
    pub chunks: HashMap<u32, SceneChunk>,
    pub size_x: f64,
    pub size_y: f64,
    pub size_z: f64,
    /// The coordinates for the lower bound of the first chunk, used to calculate which chunk a coordinate is in.
    pub chunk_starts: Vector3<f64>,
}

impl<C> Chunks<C>
where
    C: Unsigned + Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    /// TODO: move `coords_to_chunk_index` logic here
    /// This is currently just an alias for the `coords_to_chunk_index` function.
    ///
    /// # Example
    /// ```
    /// use typenum::U10;
    /// use demo::chunk::Chunks;
    /// use std::collections::HashMap;
    /// use generic_array::GenericArray;
    /// use nalgebra::Vector3;
    ///
    /// let chunks: Chunks<U10> = Chunks {
    ///     set_chunks: GenericArray::default(),
    ///     chunks: HashMap::new(),
    ///     size_x: 0.1f64,
    ///     size_y: 0.1f64,
    ///     size_z: 0.1f64,
    ///     chunk_starts: Vector3::new(0f64, 0f64, 0f64),
    /// };
    /// assert_eq!((0, 0, 0), chunks.coords_to_chunk_index(&Vector3::new(0f64, 0f64, 0f64)));
    /// assert_eq!((1, 1, 1), chunks.coords_to_chunk_index(&Vector3::new(0.1f64, 0.11f64, 0.13f64)));
    /// assert_eq!((9, 9, 8), chunks.coords_to_chunk_index(&Vector3::new(0.9f64, 0.98f64, 0.82f64)));
    /// ```
    pub fn coords_to_chunk_index(&self, coords: &Vector3<f64>) -> (u32, u32, u32) {
        coords_to_chunk_index(coords, self)
    }

    /// Get the array/map key for the chunk corresponding to the given coordinates.
    /// The key is calculated as x * C^2 + y * C + z, with x, y and z being the chunk indices
    /// corresponding to the coordinates.
    ///
    /// # Example
    /// ```
    /// use typenum::U10;
    /// use demo::chunk::Chunks;
    /// use std::collections::HashMap;
    /// use generic_array::GenericArray;
    /// use nalgebra::Vector3;
    ///
    /// let chunks: Chunks<U10> = Chunks {
    ///     set_chunks: GenericArray::default(),
    ///     chunks: HashMap::new(),
    ///     size_x: 0.1f64,
    ///     size_y: 0.1f64,
    ///     size_z: 0.1f64,
    ///     chunk_starts: Vector3::new(0f64, 0f64, 0f64),
    /// };
    /// assert_eq!(0, chunks.key_for_coordinates(&Vector3::new(0f64, 0f64, 0f64)));
    /// assert_eq!(111, chunks.key_for_coordinates(&Vector3::new(0.1f64, 0.11f64, 0.13f64)));
    /// assert_eq!(998, chunks.key_for_coordinates(&Vector3::new(0.9f64, 0.98f64, 0.82f64)));
    /// ```
    pub fn key_for_coordinates(&self, coords: &Vector3<f64>) -> u32 {
        let index = coords_to_chunk_index(coords, self);
        self.key_for_index(index.0, index.1, index.2)
    }

    /// Get the array/map key for the chunk corresponding to the given chunk index.
    /// The key is calculated as x * C^2 + y * C + z.
    ///
    /// # Example
    /// ```
    /// use typenum::U10;
    /// use demo::chunk::Chunks;
    /// use std::collections::HashMap;
    /// use generic_array::GenericArray;
    /// use nalgebra::Vector3;
    ///
    /// let chunks: Chunks<U10> = Chunks {
    ///     set_chunks: GenericArray::default(),
    ///     chunks: HashMap::new(),
    ///     size_x: 0.1f64,
    ///     size_y: 0.1f64,
    ///     size_z: 0.1f64,
    ///     chunk_starts: Vector3::new(0f64, 0f64, 0f64),
    /// };
    /// assert_eq!(0, chunks.key_for_index(0, 0, 0));
    /// assert_eq!(111, chunks.key_for_index(1, 1, 1));
    /// assert_eq!(999, chunks.key_for_index(9, 9, 9));
    /// ```
    pub fn key_for_index(&self, x: u32, y: u32, z: u32) -> u32 {
        x * C::to_u32() * C::to_u32() + y * C::to_u32() + z
    }

    /// Add an object with the given index to the chunk at the given key position.
    /// This will set the according `set_chunks` bit to true and, if necessary,
    /// add the chunk to the `chunks` map.
    ///
    /// # Example
    /// ```
    /// use typenum::U10;
    /// use demo::chunk::{Chunks, SceneChunk, TimedChunkEntry};
    /// use std::collections::HashMap;
    /// use generic_array::GenericArray;
    /// use nalgebra::Vector3;
    ///
    /// let mut chunks: Chunks<U10> = Chunks {
    ///     set_chunks: GenericArray::default(),
    ///     chunks: HashMap::new(),
    ///     size_x: 0.1f64,
    ///     size_y: 0.1f64,
    ///     size_z: 0.1f64,
    ///     chunk_starts: Vector3::new(0f64, 0f64, 0f64),
    /// };
    ///
    /// chunks.add_surface_at(0, 0, 0, 1, None);
    /// chunks.add_surface_at(0, 0, 0, 2, Some((10, Some(4000))));
    /// chunks.add_surface_at(0, 0, 0, 3, Some((500, None)));
    /// assert_eq!(true, chunks.set_chunks[0]);
    /// let chunk = chunks.chunks.get(&0).unwrap();
    /// assert_eq!(&SceneChunk {
    ///     receivers: vec![],
    ///     surfaces: vec![
    ///         TimedChunkEntry::Static(1),
    ///         TimedChunkEntry::Dynamic(2, 10, 4000),
    ///         TimedChunkEntry::Final(3, 500),
    ///     ]
    /// }, chunk);
    /// ```
    pub fn add_surface_at(
        &mut self,
        x: u32,
        y: u32,
        z: u32,
        index: usize,
        time: Option<(u32, Option<u32>)>,
    ) {
        let key = self.key_for_index(x, y, z);
        self.set_chunks[key as usize] = true;
        let entry = create_chunk_entry(index, time);
        let chunk = self.chunks.get_mut(&key);
        if let Some(chunk) = chunk {
            chunk.surfaces.push(entry);
        } else {
            self.chunks.insert(
                key,
                SceneChunk {
                    surfaces: vec![entry],
                    receivers: vec![],
                },
            );
        }
    }

    /// Add a receiver with the given index to the chunk at the given key position.
    /// This will set the according `set_chunks` bit to true and, if necessary,
    /// add the chunk to the `chunks` map.
    ///
    /// # Example
    /// ```
    /// use typenum::U10;
    /// use demo::chunk::{Chunks, SceneChunk, TimedChunkEntry};
    /// use std::collections::HashMap;
    /// use generic_array::GenericArray;
    /// use nalgebra::Vector3;
    ///
    /// let mut chunks: Chunks<U10> = Chunks {
    ///     set_chunks: GenericArray::default(),
    ///     chunks: HashMap::new(),
    ///     size_x: 0.1f64,
    ///     size_y: 0.1f64,
    ///     size_z: 0.1f64,
    ///     chunk_starts: Vector3::new(0f64, 0f64, 0f64),
    /// };
    ///
    /// chunks.add_receiver_at(0, 0, 0, 1, None);
    /// chunks.add_receiver_at(0, 1, 1, 2, Some((10, Some(4000))));
    /// chunks.add_receiver_at(0, 1, 1, 3, Some((700, None)));
    /// assert_eq!(true, chunks.set_chunks[0]);
    /// let chunk = chunks.chunks.get(&0).unwrap();
    /// assert_eq!(&SceneChunk {
    ///     surfaces: vec![],
    ///     receivers: vec![
    ///         TimedChunkEntry::Static(1),
    ///     ]
    /// }, chunk);
    /// let chunk = chunks.chunks.get(&11).unwrap();
    /// assert_eq!(&SceneChunk {
    ///     surfaces: vec![],
    ///     receivers: vec![
    ///         TimedChunkEntry::Dynamic(2, 10, 4000),
    ///         TimedChunkEntry::Final(3, 700),
    ///     ]
    /// }, chunk);
    /// ```
    pub fn add_receiver_at(
        &mut self,
        x: u32,
        y: u32,
        z: u32,
        index: usize,
        time: Option<(u32, Option<u32>)>,
    ) {
        let key = self.key_for_index(x, y, z);
        self.set_chunks[key as usize] = true;
        let entry = create_chunk_entry(index, time);
        let chunk = self.chunks.get_mut(&key);
        if let Some(chunk) = chunk {
            chunk.receivers.push(entry);
        } else {
            self.chunks.insert(
                key,
                SceneChunk {
                    surfaces: vec![],
                    receivers: vec![entry],
                },
            );
        }
    }

    /// Check whether the given chunk holds any data by checking the
    /// `set_chunks` bit.
    ///
    /// # Example
    /// ```
    /// use typenum::U10;
    /// use demo::chunk::{Chunks, SceneChunk, TimedChunkEntry};
    /// use std::collections::HashMap;
    /// use generic_array::GenericArray;
    /// use nalgebra::Vector3;
    ///
    /// let mut chunks: Chunks<U10> = Chunks {
    ///     set_chunks: GenericArray::default(),
    ///     chunks: HashMap::new(),
    ///     size_x: 0.1f64,
    ///     size_y: 0.1f64,
    ///     size_z: 0.1f64,
    ///     chunk_starts: Vector3::new(0f64, 0f64, 0f64),
    /// };
    ///
    /// chunks.add_receiver_at(0, 0, 0, 1, None);
    /// assert_eq!(true, chunks.is_chunk_set(0));
    /// assert_eq!(false, chunks.is_chunk_set(98));
    /// ```
    pub fn is_chunk_set(&self, key: usize) -> bool {
        self.set_chunks[key]
    }

    /// Retrieve all receiver and surface indices within the chunk with the given key
    /// at the given time.
    pub fn objects_at_key_and_time(
        &self,
        key: u32,
        time_entry: u32,
        time_exit: u32,
        loop_duration: Option<u32>,
    ) -> (Vec<usize>, Vec<usize>) {
        self.chunks.get(&key).map_or_else(
            || (vec![], vec![]),
            |chunk| chunk.objects_at_time(time_entry, time_exit, loop_duration),
        )
    }
}

/// Create the `TimedChunkEntry` for the given index and time.
const fn create_chunk_entry(index: usize, time: Option<(u32, Option<u32>)>) -> TimedChunkEntry {
    match time {
        Some((enter, exit)) => match exit {
            Some(exit) => TimedChunkEntry::Dynamic(index, enter, exit),
            None => TimedChunkEntry::Final(index, enter),
        },
        None => TimedChunkEntry::Static(index),
    }
}

impl Scene {
    /// Calculate the chunks for this scene.
    ///
    /// The amount of chunks calculated is determined by C - a higher amount will provide more accuracy
    /// when using the chunks (i.e. less needless intersection calculations), but will be more expensive to calculate.
    /// A balance for what amount of chunks is worthwhile needs to be determined via benchmarking.
    ///
    /// Chunks are split up in equal parts between the minimum and maximum x/y/z value that appears in the scene.
    /// To avoid edge-case issues, the scene's maximum bounds are padded by 0.1 in each direction.
    ///
    /// For surfaces and receivers, the chunks they are in are calculated on a per-keyframe-pair basis:
    /// Each keyframe pair (so the first and second, second and third, ...) is iterated over individually, calculating
    /// which chunks they are in and when.
    /// This avoids excessive chunking in cases where, for example, a surface moves along an L-shaped path.
    pub fn chunks<C>(&self) -> Chunks<C>
    where
        C: Unsigned + Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        let number_of_chunks = C::to_i32() as u16;
        let (mut min_bounds, mut max_bounds) = self.maximum_bounds();
        min_bounds.add_scalar_mut(-0.1);
        max_bounds.add_scalar_mut(0.1);
        let (x_chunk_size, y_chunk_size, z_chunk_size) =
            calculate_chunk_size(&min_bounds, &max_bounds, number_of_chunks);

        let mut result: Chunks<C> = Chunks {
            set_chunks: GenericArray::default(),
            chunks: HashMap::new(),
            size_x: x_chunk_size,
            size_y: y_chunk_size,
            size_z: z_chunk_size,
            chunk_starts: min_bounds,
        };

        for (index, surface) in self.surfaces.iter().enumerate() {
            add_surface_to_chunks(surface, &mut result, index, self);
        }
        add_receiver_to_chunks(&self.receiver, &mut result, self);

        result
    }
}

/// Calculate the chunk size from the given maximum bounds and
/// desired number of chunks.
fn calculate_chunk_size(
    min_coords: &Vector3<f64>,
    max_coords: &Vector3<f64>,
    number: u16,
) -> (f64, f64, f64) {
    (
        single_chunk_size(min_coords.x, max_coords.x, number),
        single_chunk_size(min_coords.y, max_coords.y, number),
        single_chunk_size(min_coords.z, max_coords.z, number),
    )
}

/// Calculate the chunk size between the given min/max coordinate. If it is 0,
/// use 0.1 instead to avoid zero-width chunks. This shouldn't be able to happen.
fn single_chunk_size(min: f64, max: f64, number: u16) -> f64 {
    let result = (max - min) / f64::from(number);
    if result <= 0f64 {
        return 0.1f64;
    }
    result
}

/// Add the given surface to the chunks.
///
/// For already interpolated surfaces, this will simply add it to each chunk touched by the
/// box created by its coordinates' bounds, with no time constraint.
///
/// For keyframe surfaces, this will iterate over each pair of keyframes and add them to the according
/// chunks following the logic from `add_keyframe_pair_to_chunks`.
fn add_surface_to_chunks<const N: usize, C>(
    surface: &Surface<N>,
    chunks: &mut Chunks<C>,
    index: usize,
    scene: &Scene,
) where
    C: Unsigned + Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    match surface {
        Surface::Interpolated(coordinates, _time, _material) => {
            add_coordinate_slice_to_chunks(coordinates, index, chunks, None);
        }
        Surface::Keyframes(keyframes, _material) => {
            let first_keyframe = &keyframes[0];
            if first_keyframe.time != 0 {
                add_coordinate_slice_to_chunks(
                    &first_keyframe.coords,
                    index,
                    chunks,
                    Some((0, Some(first_keyframe.time))),
                );
            }
            keyframes.windows(2).for_each(|pair| {
                add_surface_keyframe_pair_to_chunks(pair[0], &pair[1], chunks, index);
            });
            let last_keyframe = keyframes.last().unwrap();
            // when looping, the last keyframe counts until the end of the scene. Otherwise, it's a final keyframe
            let last_time = scene.loop_duration;
            add_coordinate_slice_to_chunks(
                &last_keyframe.coords,
                index,
                chunks,
                Some((last_keyframe.time, last_time)),
            );
        }
    }
}

/// Add the given receiver to the chunks.
///
/// For already interpolated receiver, this will simply add it to each chunk touched by the
/// sphere represented by the receiver, with no time constraint.
///
/// For keyframe receivers, this will iterate over each pair of keyframes and add them to the according
/// chunks following the logic from `add_keyframe_pair_to_chunks`.
fn add_receiver_to_chunks<C>(receiver: &Receiver, chunks: &mut Chunks<C>, scene: &Scene)
where
    C: Unsigned + Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    match receiver {
        Receiver::Interpolated(coordinates, radius, _time) => {
            add_sphere_to_chunks(coordinates, *radius, 0, chunks, None);
        }
        Receiver::Keyframes(keyframes, radius) => {
            let first_keyframe = &keyframes[0];
            if first_keyframe.time != 0 {
                add_sphere_to_chunks(
                    &first_keyframe.coords,
                    *radius,
                    0,
                    chunks,
                    Some((0, Some(first_keyframe.time))),
                );
            }
            keyframes.windows(2).for_each(|pair| {
                add_sphere_keyframe_pair_to_chunks(pair[0], &pair[1], *radius, chunks, 0);
            });
            let last_keyframe = keyframes.last().unwrap();
            // when looping, the last keyframe counts until the end of the scene. Otherwise, it's a final keyframe
            let last_time = scene.loop_duration;
            add_sphere_to_chunks(
                &last_keyframe.coords,
                *radius,
                0,
                chunks,
                Some((last_keyframe.time, last_time)),
            );
        }
    }
}

/// Calculate when the object described by the two given keyframes first and last enters
/// which chunks, then add it to them accordingly.
///
/// This works by starting out in the middle between the first and second keyframe
/// and halving the distance to the first keyframe until the first and middle keyframe
/// fill the same chunks. Then the middle keyframe's time is incremented until it no longer fits within the same chunk boundaries,
/// and the resulting time and chunks are written accordingly.
///
/// This process is repeated until the second keyframe's time is reached.
fn add_surface_keyframe_pair_to_chunks<const N: usize, C>(
    mut first: SurfaceKeyframe<N>,
    second: &SurfaceKeyframe<N>,
    chunks: &mut Chunks<C>,
    index: usize,
) where
    C: Unsigned + Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    let mut chunks_at_first = chunk_bounds(&first.coords, chunks);
    let mut time = first.time;
    while time < second.time {
        time = time.average_floor(&second.time);
        let mut keyframe_middle =
            interpolation::interpolate_two_surface_keyframes(&first, second, time).unwrap();

        let mut chunks_at_middle = chunk_bounds(&keyframe_middle, chunks);
        while chunks_at_middle != chunks_at_first {
            time = time.average_floor(&first.time);
            keyframe_middle =
                interpolation::interpolate_two_surface_keyframes(&first, second, time).unwrap();
            chunks_at_middle = chunk_bounds(&keyframe_middle, chunks);
        }

        // potential optimisation: if we step here often, do increments by 10 or 100, then decrement again by an order of magnitude lower
        while chunks_at_middle == chunks_at_first && time < second.time {
            time += 1;
            keyframe_middle =
                interpolation::interpolate_two_surface_keyframes(&first, second, time).unwrap();
            chunks_at_middle = chunk_bounds(&keyframe_middle, chunks);
        }

        add_coordinate_slice_to_chunks(
            &first.coords,
            index,
            chunks,
            Some((first.time, Some(time - 1))),
        );

        first = SurfaceKeyframe {
            coords: keyframe_middle,
            time,
        };
        chunks_at_first = chunks_at_middle;
    }
}

/// Calculate when the receiver described by the two given keyframes first and last enters
/// which chunks, then add it to them accordingly.
///
/// This works by starting out in the middle between the first and second keyframe
/// and halving the distance to the first keyframe until the first and middle keyframe
/// fill the same chunks. Then the middle keyframe's time is incremented until it no longer fits within the same chunk boundaries,
/// and the resulting time and chunks are written accordingly.
///
/// This process is repeated until the second keyframe's time is reached.
///
/// The chunk boundaries are simplified as a box around the receiver's sphere - in most practical uses, the receiver will be orders of magnitude
/// smaller than the chunks, so there's no major accuracy loss by simplifying to a box.
fn add_sphere_keyframe_pair_to_chunks<C>(
    mut first: CoordinateKeyframe,
    second: &CoordinateKeyframe,
    radius: f64,
    chunks: &mut Chunks<C>,
    index: usize,
) where
    C: Unsigned + Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    let mut chunks_at_first = sphere_chunk_bounds(&first.coords, radius, chunks);
    let mut time = first.time;
    while time < second.time {
        time = time.average_floor(&second.time);
        let mut keyframe_middle =
            interpolation::interpolate_two_coordinate_keyframes(&first, second, time).unwrap();

        let mut chunks_at_middle = sphere_chunk_bounds(&keyframe_middle, radius, chunks);
        while chunks_at_middle != chunks_at_first {
            time = time.average_floor(&first.time);
            keyframe_middle =
                interpolation::interpolate_two_coordinate_keyframes(&first, second, time).unwrap();
            chunks_at_middle = sphere_chunk_bounds(&keyframe_middle, radius, chunks);
        }

        // potential optimisation: if we step here often, do increments by 10 or 100, then decrement again by an order of magnitude lower
        while chunks_at_middle == chunks_at_first && time < second.time {
            time += 1;
            keyframe_middle =
                interpolation::interpolate_two_coordinate_keyframes(&first, second, time).unwrap();
            chunks_at_middle = sphere_chunk_bounds(&keyframe_middle, radius, chunks);
        }

        add_sphere_to_chunks(
            &first.coords,
            radius,
            index,
            chunks,
            Some((first.time, Some(time - 1))),
        );

        first = CoordinateKeyframe {
            coords: keyframe_middle,
            time,
        };
        chunks_at_first = chunks_at_middle;
    }
}

/// Add the object described by the given index to all chunks touched by the
/// box formed by the given coordinate slice's maximum bounds.
fn add_coordinate_slice_to_chunks<C>(
    coordinates: &[Vector3<f64>],
    index: usize,
    chunks: &mut Chunks<C>,
    time: Option<(u32, Option<u32>)>,
) where
    C: Unsigned + Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    // possible optimisation: move along surface rather than creating a box around it
    let (min_index, max_index) = chunk_bounds(coordinates, chunks);

    for x in min_index.0..=max_index.0 {
        for y in min_index.1..=max_index.1 {
            for z in min_index.2..=max_index.2 {
                chunks.add_surface_at(x, y, z, index, time);
            }
        }
    }
}

/// Add the object described by the given index to all chunks touched by the
/// box formed by the given coordinate slice's maximum bounds.
fn add_sphere_to_chunks<C>(
    coordinates: &Vector3<f64>,
    radius: f64,
    index: usize,
    chunks: &mut Chunks<C>,
    time: Option<(u32, Option<u32>)>,
) where
    C: Unsigned + Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    // possible optimisation: move along surface rather than creating a box around it
    let (min_index, max_index) = sphere_chunk_bounds(coordinates, radius, chunks);

    for x in min_index.0..=max_index.0 {
        for y in min_index.1..=max_index.1 {
            for z in min_index.2..=max_index.2 {
                chunks.add_receiver_at(x, y, z, index, time);
            }
        }
    }
}

/// Calculate the box formed around the given sphere
/// bounds, represented as its boundaries' chunk indices.
fn sphere_chunk_bounds<C>(
    coordinates: &Vector3<f64>,
    radius: f64,
    chunks: &Chunks<C>,
) -> ((u32, u32, u32), (u32, u32, u32))
where
    C: Unsigned + Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    let mut minimum_bounds = *coordinates;
    minimum_bounds.add_scalar_mut(-radius);
    let mut maximum_bounds = *coordinates;
    maximum_bounds.add_scalar_mut(radius);
    (
        coords_to_chunk_index(&minimum_bounds, chunks),
        coords_to_chunk_index(&maximum_bounds, chunks),
    )
}

/// Calculate the box formed by the given coordinates' maximum
/// bounds, represented as its boundaries' chunk indices.
fn chunk_bounds<C>(
    coordinates: &[Vector3<f64>],
    chunks: &Chunks<C>,
) -> ((u32, u32, u32), (u32, u32, u32))
where
    C: Unsigned + Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    let coords_at_second = scene_bounds::maximum_bounds(coordinates);
    (
        coords_to_chunk_index(&coords_at_second.0, chunks),
        coords_to_chunk_index(&coords_at_second.1, chunks),
    )
}

/// Convert the given coordinates into their related chunk indices.
fn coords_to_chunk_index<C>(coords: &Vector3<f64>, chunks: &Chunks<C>) -> (u32, u32, u32)
where
    C: Unsigned + Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    (
        ((coords.x - chunks.chunk_starts.x) / chunks.size_x).floor() as u32,
        ((coords.y - chunks.chunk_starts.y) / chunks.size_y).floor() as u32,
        ((coords.z - chunks.chunk_starts.z) / chunks.size_z).floor() as u32,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use generic_array::GenericArray;
    use nalgebra::Vector3;
    use typenum::U10;

    use crate::chunk::{
        chunk_bounds, coords_to_chunk_index, create_chunk_entry, single_chunk_size,
        sphere_chunk_bounds, TimedChunkEntry,
    };

    use super::{calculate_chunk_size, Chunks};

    fn empty_chunks() -> Chunks<U10> {
        Chunks {
            set_chunks: GenericArray::default(),
            chunks: HashMap::new(),
            size_x: 0.2f64,
            size_y: 0.2f64,
            size_z: 0.2f64,
            chunk_starts: Vector3::new(-1f64, -1f64, -1f64),
        }
    }

    #[test]
    fn static_chunk_entry_object_index() {
        let entry = TimedChunkEntry::Static(1094);
        assert_eq!(1094, entry.object_index());
    }

    #[test]
    fn dynamic_chunk_entry_object_index() {
        let entry = TimedChunkEntry::Dynamic(299, 1000, 6000);
        assert_eq!(299, entry.object_index());
    }

    #[test]
    fn final_chunk_entry_object_index() {
        let entry = TimedChunkEntry::Final(4901, 6000);
        assert_eq!(4901, entry.object_index());
    }


    #[test]
    fn create_chunk_entry_static_dynamic_and_final() {
        assert_eq!(TimedChunkEntry::Static(12), create_chunk_entry(12, None));
        assert_eq!(
            TimedChunkEntry::Dynamic(12, 0, 1000),
            create_chunk_entry(12, Some((0, Some(1000))))
        );
        assert_eq!(
            TimedChunkEntry::Final(12, 19000),
            create_chunk_entry(12, Some((19000, None)))
        );
    }

    #[test]
    fn calculate_chunk_size_empty() {
        assert_eq!(
            (0.1f64, 0.1f64, 0.1f64),
            calculate_chunk_size(
                &Vector3::new(0f64, 0f64, 0f64),
                &Vector3::new(0f64, 0f64, 0f64),
                10,
            )
        );
    }

    #[test]
    fn calculate_chunk_size_normal_scene() {
        assert_eq!(
            (2f64, 2f64, 4f64),
            calculate_chunk_size(
                &Vector3::new(-20f64, 10f64, 10f64),
                &Vector3::new(0f64, 30f64, 50f64),
                10,
            )
        );
    }

    #[test]
    fn single_chunk_size_empty() {
        assert_eq!(0.1f64, single_chunk_size(0f64, 0f64, u16::MAX));
    }

    #[test]
    fn single_chunk_size_normal() {
        assert_eq!(2.5f64, single_chunk_size(0f64, 50f64, 20));
    }

    #[test]
    fn single_chunk_size_giant() {
        assert_eq!(20f64, single_chunk_size(-100_000f64, 100_000f64, 10000));
    }

    // TODO
    // add_surface_keyframe_pair_to_chunks
    // add_sphere_keyframe_pair_to_chunks
    // add_coordinate_slice_to_chunks
    // add_sphere_to_chunks

    #[test]
    fn sphere_chunk_bounds_full_chunk() {
        let chunks = empty_chunks();
        assert_eq!(
            ((0, 0, 0), (9, 9, 9)),
            sphere_chunk_bounds(&Vector3::new(0f64, 0f64, 0f64), 0.9f64, &chunks)
        );
    }

    #[test]
    fn sphere_chunk_bounds_partial() {
        let chunks = empty_chunks();
        assert_eq!(
            ((3, 2, 3), (4, 4, 4)),
            sphere_chunk_bounds(&Vector3::new(-0.2f64, -0.3f64, -0.2f64), 0.15f64, &chunks)
        );
    }

    #[test]
    fn chunk_bounds_full_chunk() {
        let chunks = empty_chunks();
        assert_eq!(
            ((0, 0, 0), (9, 9, 9)),
            chunk_bounds(
                &[
                    Vector3::new(-1f64, -1f64, -1f64),
                    Vector3::new(0.9f64, 0.99f64, 0.999f64)
                ],
                &chunks
            )
        );
    }

    #[test]
    fn chunk_bounds_partial() {
        let chunks = empty_chunks();
        assert_eq!(
            ((6, 5, 5), (9, 5, 6)),
            chunk_bounds(
                &[
                    Vector3::new(0.3f64, 0.2f64, 0.3f64),
                    Vector3::new(0.9f64, 0.1f64, 0.2f64)
                ],
                &chunks
            )
        );
    }

    #[test]
    fn lower_bound_coords_to_chunk_index() {
        let chunks = empty_chunks();
        assert_eq!(
            (0, 0, 0),
            coords_to_chunk_index(&Vector3::new(-1f64, -1f64, -1f64), &chunks)
        )
    }

    #[test]
    fn middle_coords_to_chunk_index() {
        let chunks = empty_chunks();
        assert_eq!(
            (5, 5, 5),
            coords_to_chunk_index(&Vector3::new(0f64, 0f64, 0f64), &chunks)
        )
    }

    #[test]
    fn random_coords_to_chunk_index() {
        let chunks = empty_chunks();
        assert_eq!(
            (3, 6, 5),
            coords_to_chunk_index(&Vector3::new(-0.3f64, 0.4f64, 0.1f64), &chunks)
        )
    }

    #[test]
    fn near_upper_bound_coords_to_chunk_index() {
        let chunks = empty_chunks();
        assert_eq!(
            (9, 9, 9),
            coords_to_chunk_index(
                &Vector3::new(0.9999f64, 0.9999999f64, 0.9999999f64),
                &chunks
            )
        )
    }
}
