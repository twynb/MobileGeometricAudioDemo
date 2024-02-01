use generic_array::{ArrayLength, GenericArray};
use num_integer::Average;
use std::collections::HashMap;
use std::ops::Mul;
use typenum::{operator_aliases::Cube, Unsigned};

use crate::{
    interpolation,
    scene::{CoordinateKeyframe, Coordinates, Receiver, Scene, Surface, SurfaceKeyframe},
    scene_bounds, test_utils,
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

/// A chunk within the scene. Chunks hold a vector of [TimedChunkEntry] entries for
/// surfaces and receivers that are inside the chunk at some point in the scene.
#[derive(Clone, Debug)]
pub struct SceneChunk {
    pub surfaces: Vec<TimedChunkEntry>,
    pub receivers: Vec<TimedChunkEntry>,
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
pub struct Chunks<C: Unsigned>
where
    C: Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    /// An array with booleans indicating whether the given chunk has any data whatsoever.
    pub set_chunks: GenericArray<bool, Cube<C>>,
    /// The map of chunks holding actual data.
    pub chunks: HashMap<u32, SceneChunk>,
    pub size_x: f32,
    pub size_y: f32,
    pub size_z: f32,
    /// The coordinates for the lower bound of the first chunk, used to calculate which chunk a coordinate is in.
    pub chunk_starts: Coordinates,
}

impl<C: Unsigned> Chunks<C>
where
    C: Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    /// Get the array/map key for the chunk corresponding to the given chunk index.
    /// The key is calculated as x * C^2 + y * C + z.
    ///
    /// # Example
    /// ```
    /// use typenum::U10;
    /// use demo::chunk::Chunks;
    /// use demo::scene::Coordinates;
    /// use std::collections::HashMap;
    /// use generic_array::GenericArray;
    ///
    /// let chunks: Chunks<U10> = Chunks {
    ///     set_chunks: GenericArray::default(),
    ///     chunks: HashMap::new(),
    ///     size_x: 0.1f32,
    ///     size_y: 0.1f32,
    ///     size_z: 0.1f32,
    ///     chunk_starts: Coordinates::at(0f32, 0f32, 0f32),
    /// };
    /// assert_eq!(0, chunks.key_for_coordinates(0, 0, 0));
    /// assert_eq!(111, chunks.key_for_coordinates(1, 1, 1));
    /// assert_eq!(999, chunks.key_for_coordinates(9, 9, 9));
    /// ```
    pub fn key_for_coordinates(&self, x: u32, y: u32, z: u32) -> u32 {
        x * C::to_u32() * C::to_u32() + y * C::to_u32() + z
    }

    /// Add an object with the given index to the chunk at the given key position.
    /// This will set the according set_chunks bit to true and, if necessary,
    /// add the chunk to the chunks map.
    ///
    /// # Example
    /// ```
    /// use typenum::U10;
    /// use demo::chunk::{Chunks, SceneChunk, TimedChunkEntry};
    /// use demo::scene::Coordinates;
    /// use std::collections::HashMap;
    /// use generic_array::GenericArray;
    ///
    /// let mut chunks: Chunks<U10> = Chunks {
    ///     set_chunks: GenericArray::default(),
    ///     chunks: HashMap::new(),
    ///     size_x: 0.1f32,
    ///     size_y: 0.1f32,
    ///     size_z: 0.1f32,
    ///     chunk_starts: Coordinates::at(0f32, 0f32, 0f32),
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
        let key = self.key_for_coordinates(x, y, z);
        self.set_chunks[key as usize] = true;
        let entry = create_entry(index, time);
        let chunk = self.chunks.get_mut(&key);
        if chunk.is_some() {
            chunk.unwrap().surfaces.push(entry);
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
    /// This will set the according set_chunks bit to true and, if necessary,
    /// add the chunk to the chunks map.
    ///
    /// # Example
    /// ```
    /// use typenum::U10;
    /// use demo::chunk::{Chunks, SceneChunk, TimedChunkEntry};
    /// use demo::scene::Coordinates;
    /// use std::collections::HashMap;
    /// use generic_array::GenericArray;
    ///
    /// let mut chunks: Chunks<U10> = Chunks {
    ///     set_chunks: GenericArray::default(),
    ///     chunks: HashMap::new(),
    ///     size_x: 0.1f32,
    ///     size_y: 0.1f32,
    ///     size_z: 0.1f32,
    ///     chunk_starts: Coordinates::at(0f32, 0f32, 0f32),
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
        let key = self.key_for_coordinates(x, y, z);
        self.set_chunks[key as usize] = true;
        let entry = create_entry(index, time);
        let chunk = self.chunks.get_mut(&key);
        if chunk.is_some() {
            chunk.unwrap().receivers.push(entry);
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
}

/// Create the TimedChunkEntry for the given index and time.
fn create_entry(index: usize, time: Option<(u32, Option<u32>)>) -> TimedChunkEntry {
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
    pub fn chunks<C: Unsigned>(&self) -> Chunks<C>
    where
        C: Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        let number_of_chunks = C::to_i32() as u16;
        let (mut min_bounds, mut max_bounds) = self.maximum_bounds();
        min_bounds.add(-0.1);
        max_bounds.add(0.1);
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
            add_surface_to_chunks(surface, &mut result, index);
        }
        add_receiver_to_chunks(&self.receiver, &mut result);

        result
    }
}

/// Calculate the chunk size from the given maximum bounds and
/// desired number of chunks.
fn calculate_chunk_size(
    min_coords: &Coordinates,
    max_coords: &Coordinates,
    number: u16,
) -> (f32, f32, f32) {
    (
        single_chunk_size(min_coords.x, max_coords.x, number),
        single_chunk_size(min_coords.y, max_coords.y, number),
        single_chunk_size(min_coords.z, max_coords.z, number),
    )
}

/// Calculate the chunk size between the given min/max coordinate. If it is 0,
/// use 0.1 instead to avoid zero-width chunks. This shouldn't be able to happen.
fn single_chunk_size(min: f32, max: f32, number: u16) -> f32 {
    let result = (max - min) / number as f32;
    if result <= 0f32 {
        return 0.1f32;
    }
    result
}

/// Add the given surface to the chunks.
///
/// For already interpolated surfaces, this will simply add it to each chunk touched by the
/// box created by its coordinates' bounds, with no time constraint.
///
/// For keyframe surfaces, this will iterate over each pair of keyframes and add them to the according
/// chunks following the logic from [add_keyframe_pair_to_chunks].
fn add_surface_to_chunks<const N: usize, C: Unsigned>(
    surface: &Surface<N>,
    chunks: &mut Chunks<C>,
    index: usize,
) where
    C: Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    match surface {
        Surface::Interpolated(coordinates, _time) => {
            add_coordinate_slice_to_chunks(coordinates, index, chunks, None);
        }
        Surface::Keyframes(keyframes) => {
            keyframes.windows(2).for_each(|pair| {
                add_surface_keyframe_pair_to_chunks(pair[0].clone(), &pair[1], chunks, index);
            });
            let last_keyframe = keyframes.last().unwrap();
            add_coordinate_slice_to_chunks(
                &last_keyframe.coords,
                index,
                chunks,
                Some((last_keyframe.time, None)),
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
/// chunks following the logic from [add_keyframe_pair_to_chunks].
fn add_receiver_to_chunks<C: Unsigned>(receiver: &Receiver, chunks: &mut Chunks<C>)
where
    C: Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    match receiver {
        Receiver::Interpolated(coordinates, radius, _time) => {
            add_sphere_to_chunks(coordinates, *radius, 0, chunks, None);
        }
        Receiver::Keyframes(keyframes, radius) => {
            keyframes.windows(2).for_each(|pair| {
                add_sphere_keyframe_pair_to_chunks(pair[0].clone(), &pair[1], *radius, chunks, 0)
            });
            let last_keyframe = keyframes.last().unwrap();
            add_sphere_to_chunks(&last_keyframe.coords, *radius, 0, chunks, Some((last_keyframe.time, None)));
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
fn add_surface_keyframe_pair_to_chunks<const N: usize, C: Unsigned>(
    mut first: SurfaceKeyframe<N>,
    second: &SurfaceKeyframe<N>,
    chunks: &mut Chunks<C>,
    index: usize,
) where
    C: Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    let mut chunks_at_first = chunk_bounds(&first.coords, chunks);
    let mut time = first.time;
    while time < second.time {
        time = time.average_ceil(&second.time);
        let mut keyframe_middle =
            interpolation::interpolate_two_surface_keyframes(&first, &second, time).unwrap();

        let mut chunks_at_middle = chunk_bounds(&keyframe_middle, chunks);
        while chunks_at_middle != chunks_at_first {
            time = time.average_ceil(&first.time);
            keyframe_middle =
                interpolation::interpolate_two_surface_keyframes(&first, &second, time).unwrap();
            chunks_at_middle = chunk_bounds(&keyframe_middle, chunks);
        }

        // potential optimisation: if we step here often, do increments by 10 or 100, then decrement again by an order of magnitude lower
        while chunks_at_middle == chunks_at_first && time < second.time {
            time += 1;
            keyframe_middle =
                interpolation::interpolate_two_surface_keyframes(&first, &second, time).unwrap();
            chunks_at_middle = chunk_bounds(&keyframe_middle, chunks);
        }

        add_coordinate_slice_to_chunks(
            &keyframe_middle,
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
fn add_sphere_keyframe_pair_to_chunks<C: Unsigned>(
    mut first: CoordinateKeyframe,
    second: &CoordinateKeyframe,
    radius: f32,
    chunks: &mut Chunks<C>,
    index: usize,
) where
    C: Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    let mut chunks_at_first = sphere_chunk_bounds(&first.coords, radius, chunks);
    let mut time = first.time;
    while time < second.time {
        time = time.average_floor(&second.time);
        let mut keyframe_middle =
            interpolation::interpolate_two_coordinate_keyframes(&first, &second, time).unwrap();

        let mut chunks_at_middle = sphere_chunk_bounds(&keyframe_middle, radius, chunks);
        while chunks_at_middle != chunks_at_first {
            time = time.average_floor(&first.time);
            keyframe_middle =
                interpolation::interpolate_two_coordinate_keyframes(&first, &second, time).unwrap();
            chunks_at_middle = sphere_chunk_bounds(&keyframe_middle, radius, chunks);
        }

        // potential optimisation: if we step here often, do increments by 10 or 100, then decrement again by an order of magnitude lower
        while chunks_at_middle == chunks_at_first && time < second.time {
            time += 1;
            keyframe_middle =
                interpolation::interpolate_two_coordinate_keyframes(&first, &second, time).unwrap();
            chunks_at_middle = sphere_chunk_bounds(&keyframe_middle, radius, chunks);
        }

        add_sphere_to_chunks(
            &keyframe_middle,
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
fn add_coordinate_slice_to_chunks<C: Unsigned>(
    coordinates: &[Coordinates],
    index: usize,
    chunks: &mut Chunks<C>,
    time: Option<(u32, Option<u32>)>,
) where
    C: Mul<C>,
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
fn add_sphere_to_chunks<C: Unsigned>(
    coordinates: &Coordinates,
    radius: f32,
    index: usize,
    chunks: &mut Chunks<C>,
    time: Option<(u32, Option<u32>)>,
) where
    C: Mul<C>,
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
fn sphere_chunk_bounds<C: Unsigned>(
    coordinates: &Coordinates,
    radius: f32,
    chunks: &Chunks<C>,
) -> ((u32, u32, u32), (u32, u32, u32))
where
    C: Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    let minimum_bounds = Coordinates {
        x: coordinates.x - radius,
        y: coordinates.y - radius,
        z: coordinates.z - radius,
    };
    let maximum_bounds = Coordinates {
        x: coordinates.x + radius,
        y: coordinates.y + radius,
        z: coordinates.z + radius,
    };
    (
        coords_to_chunk_index(&minimum_bounds, chunks),
        coords_to_chunk_index(&maximum_bounds, chunks),
    )
}

/// Calculate the box formed by the given coordinates' maximum
/// bounds, represented as its boundaries' chunk indices.
fn chunk_bounds<C: Unsigned>(
    coordinates: &[Coordinates],
    chunks: &Chunks<C>,
) -> ((u32, u32, u32), (u32, u32, u32))
where
    C: Mul<C>,
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
fn coords_to_chunk_index<C: Unsigned>(coords: &Coordinates, chunks: &Chunks<C>) -> (u32, u32, u32)
where
    C: Mul<C>,
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
mod tests {}
