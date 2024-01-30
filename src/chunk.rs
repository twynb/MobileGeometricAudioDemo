use generic_array::{ArrayLength, GenericArray};
use num_integer::Average;
use std::collections::HashMap;
use std::ops::Mul;
use typenum::{operator_aliases::Cube, Unsigned};

use crate::{
    interpolation,
    scene::{Coordinates, Scene, Surface, SurfaceKeyframe},
    scene_bounds,
};

/// A single chunk entry. Chunk entries are either static
/// (i.e. they just hold an object index that stays in this chunk for
/// the entirety of the scene) or dynamic (i.e. they also hold timestamps
/// for when the object enters/exits the chunk. The timestamp is inclusive,
/// meaning that at the last timestamp, the object still is within the chunk).
#[derive(Clone, Debug, PartialEq)]
enum TimedChunkEntry {
    Dynamic(usize, u32, u32),
    Static(usize),
}

/// A chunk within the scene. Chunks hold a vector of [TimedChunkEntry] entries for
/// surfaces and receivers that are inside the chunk at some point in the scene.
#[derive(Clone, Debug, PartialEq)]
struct SceneChunk {
    surfaces: Vec<TimedChunkEntry>,
    receivers: Vec<TimedChunkEntry>,
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
    set_chunks: GenericArray<bool, Cube<C>>,
    /// The map of chunks holding actual data.
    chunks: HashMap<u32, SceneChunk>,
    size_x: f32,
    size_y: f32,
    size_z: f32,
    /// The coordinates for the lower bound of the first chunk, used to calculate which chunk a coordinate is in.
    chunk_starts: Coordinates,
}

impl<C: Unsigned> Chunks<C>
where
    C: Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    /// Add an object with the given index to the chunk at the given key position.
    /// This will set the according set_chunks bit to true and, if necessary,
    /// add the chunk to the chunks map.
    pub fn add_object_at(&mut self, key: u32, index: usize, time: Option<(u32, u32)>) {
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
    pub fn add_receiver_at(&mut self, key: u32, index: usize, time: Option<(u32, u32)>) {
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
fn create_entry(index: usize, time: Option<(u32, u32)>) -> TimedChunkEntry {
    if time.is_some() {
        TimedChunkEntry::Dynamic(index, time.unwrap().0, time.unwrap().1)
    } else {
        TimedChunkEntry::Static(index)
    }
}

impl Scene {
    /// Calculate the chunks for this scene.
    ///
    /// The amount of chunks calculated is determined by N - a higher amount will provide more accuracy
    /// when using the chunks (i.e. less needless intersection calculations), but will be more expensive to calculate.
    /// A balance for what amount of chunks is worthwhile needs to be determined via benchmarking.
    ///
    /// For surfaces and receivers, the chunks they are in are calculated on a per-keyframe-pair basis:
    /// Each keyframe pair (so the first and second, second and third, ...) is iterated over individually, calculating
    /// which chunks they are in and when.
    /// This avoids excessive chunking in cases where, for example, a surface moves along an L-shaped path.
    ///
    /// TODO: Test
    fn chunks<C: Unsigned>(&self) -> Chunks<C>
    where
        C: Mul<C>,
        <C as Mul>::Output: Mul<C>,
        <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
    {
        let number_of_chunks = C::to_i32() as u16;
        let (min_bounds, max_bounds) = self.maximum_bounds();
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

        // TODO: receiver

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
/// use 0.1 instead to avoid zero-width chunks.
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
fn add_surface_to_chunks<const N: usize, C: Unsigned>(surface: &Surface<N>, chunks: &mut Chunks<C>, index: usize)
where
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
                add_keyframe_pair_to_chunks(pair[0].clone(), &pair[1], chunks, index)
            });
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
fn add_keyframe_pair_to_chunks<const N: usize, C: Unsigned>(
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
            Some((first.time, time - 1)),
        );

        first = SurfaceKeyframe {
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
    time: Option<(u32, u32)>,
) where
    C: Mul<C>,
    <C as Mul>::Output: Mul<C>,
    <<C as Mul>::Output as Mul<C>>::Output: ArrayLength,
{
    // possible optimisation: move along surface rather than creating a box around it
    let (min_index, max_index) = chunk_bounds(coordinates, chunks);

    let mut key = (min_index.0 << 16) + (min_index.1 << 8) + min_index.2;
    for _x in min_index.0..max_index.0 {
        for _y in min_index.1..max_index.1 {
            for _z in min_index.2..max_index.2 {
                chunks.add_object_at(key, index, time);
                key += 1; // increment z part of key
            }
            key &= 0xFFFF00; // reset z part of key
            key += min_index.2;
            key += 0x100; // then increment y part of key
        }
        key &= 0xFF00FF; // reset y part of key
        key += min_index.1 << 8;
        key += 0x10000; // then increment x part of key
    }
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
        coords_to_chunk_index(&coords_at_second.0, chunks),
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
mod tests {
    use std::collections::HashMap;

    use generic_array::GenericArray;

    use crate::{chunk::{Chunks, SceneChunk, TimedChunkEntry}, scene::{Scene, Receiver, CoordinateKeyframe, Coordinates, Emitter, Surface, SurfaceKeyframe}};

    fn empty_scene() -> Scene {
        Scene {
            receiver: Receiver::Interpolated(
                Coordinates {
                        ..Default::default()
                },
                0.1,
                0
            ),
            surfaces: vec![],
            emitter: Emitter::Keyframes(
                vec![CoordinateKeyframe {
                    time: 0,
                    coords: Coordinates {
                        ..Default::default()
                    },
                }],
            ),
        }
    }

    #[test]
    fn chunks_empty_scene() {
        let scene = empty_scene();
        let result = scene.chunks::<typenum::U10>();
        assert_eq!((0.02f32, 0.02f32, 0.02f32), (result.size_x, result.size_y, result.size_z));

        let mut set_chunks: GenericArray<bool, typenum::U1000> = GenericArray::default();
        set_chunks[0] = true;
        let mut chunks: HashMap<u32, SceneChunk> = HashMap::new();
        chunks.insert(0, SceneChunk {
            surfaces: vec![],
            receivers: vec![TimedChunkEntry::Static(0)]
        });
        let expected: Chunks<typenum::U10> = Chunks {
            set_chunks,
            chunks,
            size_x: 0.1,
            size_y: 0.1,
            size_z: 0.1,
            chunk_starts: Coordinates {x: 0f32, y: 0f32, z: 0f32}
        };

        assert_eq!(expected, result);
    }

    /*
    #[test]
    fn chunks_moving_receiver_and_surfaces() {
        let scene = Scene {
            receiver: Receiver {
                keyframes: Some(vec![
                    CoordinateKeyframe {
                        time: 0,
                        coords: Coordinates {
                            x: -2f32,
                            y: -3f32,
                            z: 0f32,
                            w: 0.5f32,
                        },
                    },
                    CoordinateKeyframe {
                        time: 10,
                        coords: Coordinates {
                            x: -1f32,
                            y: 1f32,
                            z: 0f32,
                            w: 1f32,
                        },
                    },
                ]),
                index: 0,
                coordinates: None,
            },
            surfaces: vec![Surface {
                index: 0,
                coordinates: None,
                keyframes: Some(vec![
                    SurfaceKeyframe {
                        time: 0,
                        coords: [
                            Coordinates {
                                x: -1.0,
                                y: -1.0,
                                z: -2.0,
                                w: 1.0,
                            },
                            Coordinates {
                                x: 0.0,
                                y: -1.0,
                                z: -2.0,
                                w: 1.0,
                            },
                            Coordinates {
                                x: 0.0,
                                y: -2.0,
                                z: -2.0,
                                w: 1.0,
                            },
                            Coordinates {
                                x: -1.0,
                                y: -2.0,
                                z: -2.0,
                                w: 1.0,
                            },
                        ],
                    },
                    SurfaceKeyframe {
                        time: 0,
                        coords: [
                            Coordinates {
                                x: -1.0,
                                y: -1.0,
                                z: -1.0,
                                w: 1.0,
                            },
                            Coordinates {
                                x: 0.0,
                                y: -1.0,
                                z: -1.0,
                                w: 1.0,
                            },
                            Coordinates {
                                x: 0.0,
                                y: -2.0,
                                z: -1.0,
                                w: 1.0,
                            },
                            Coordinates {
                                x: -1.0,
                                y: -2.0,
                                z: -1.0,
                                w: 1.0,
                            },
                        ],
                    },
                ]),
            }],
            emitter: Emitter {
                keyframes: Some(vec![CoordinateKeyframe {
                    time: 0,
                    coords: Coordinates {
                        ..Default::default()
                    },
                }]),
                index: 0,
                coordinates: None,
            },
        };

        let (chunks, chunk_size) = scene.chunks::<10>();
        assert_eq!((0.3f32, 0.7f32, 0.1f32), chunk_size);

        let mut expected: Chunks3D<10> = array_init::array_init(|_| {
            array_init::array_init(|_2| {
                array_init::array_init(|_3| SceneChunk {
                    object_indices: vec![],
                    receiver_indices: vec![],
                })
            })
        });
        for x in 0..10 {
            for y in 0..10 {
                expected[x][y][0].receiver_indices.push(0);
            }
        }

        assert_eq!(expected, chunks);
    }
    */
}
