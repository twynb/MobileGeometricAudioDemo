use std::collections::HashMap;

use generic_array::GenericArray;

use demo::{
    chunk::{Chunks, SceneChunk, TimedChunkEntry},
    scene::{CoordinateKeyframe, Coordinates, Emitter, Receiver, Scene, Surface, SurfaceKeyframe},
    scene_builder,
};

fn empty_scene() -> Scene {
    Scene {
        receiver: Receiver::Interpolated(
            Coordinates {
                ..Default::default()
            },
            0.1,
            0,
        ),
        surfaces: vec![],
        emitter: Emitter::Keyframes(vec![CoordinateKeyframe {
            time: 0,
            coords: Coordinates {
                ..Default::default()
            },
        }]),
    }
}

fn assert_set_chunks_equal(
    set_chunks: &GenericArray<bool, typenum::U1000>,
    result: &Chunks<typenum::U10>,
) {
    for idx in 0..1000 {
        assert_eq!(
            set_chunks[idx], result.set_chunks[idx],
            "mismatch in set_chunks at index {}",
            idx
        );
    }
}

fn assert_chunks_equal(chunks: &HashMap<u32, SceneChunk>, result: &Chunks<typenum::U10>) {
    for idx in 0..1000 {
        assert_eq!(
            chunks.get(&idx),
            result.chunks.get(&idx),
            "mismatch in chunks at index {}",
            idx
        );
    }
}

#[test]
fn chunks_empty_scene() {
    let scene = empty_scene();
    let result = scene.chunks::<typenum::U10>();
    assert_eq!(
        (0.04f32, 0.04f32, 0.04f32),
        (result.size_x, result.size_y, result.size_z)
    );
    assert_eq!(
        Coordinates::at(-0.2f32, -0.2f32, -0.2f32),
        result.chunk_starts
    );

    let mut set_chunks: GenericArray<bool, typenum::U1000> = GenericArray::default();
    for x in 2..8 {
        for y in 2..8 {
            for z in 2..8 {
                set_chunks[x * 100 + y * 10 + z] = true;
            }
        }
    }
    assert_set_chunks_equal(&set_chunks, &result);

    let mut chunks: HashMap<u32, SceneChunk> = HashMap::new();
    for x in 2..8 {
        for y in 2..8 {
            for z in 2..8 {
                let key = x * 100 + y * 10 + z;
                chunks.insert(
                    key,
                    SceneChunk {
                        surfaces: vec![],
                        receivers: vec![TimedChunkEntry::Static(0)],
                    },
                );
            }
        }
    }
    assert_chunks_equal(&chunks, &result);

    let expected: Chunks<typenum::U10> = Chunks {
        set_chunks,
        chunks,
        size_x: 0.04,
        size_y: 0.04,
        size_z: 0.04,
        chunk_starts: Coordinates::at(-0.2f32, -0.2f32, -0.2f32),
    };
    assert_eq!(expected, result);
}

#[test]
fn chunks_static_scene_moving_receiver() {
    let scene = Scene {
        receiver: Receiver::Keyframes(
            vec![
                CoordinateKeyframe {
                    time: 0,
                    coords: Coordinates::at(-1f32, -1f32, -1f32),
                },
                CoordinateKeyframe {
                    time: 10,
                    coords: Coordinates::at(1f32, -1f32, 0f32),
                },
                CoordinateKeyframe {
                    time: 30,
                    coords: Coordinates::at(1f32, 1f32, 1f32),
                },
            ],
            0.1,
        ),
        surfaces: scene_builder::static_cube(
            Coordinates::at(-10f32, -10f32, -10f32),
            Coordinates::at(10f32, 10f32, 10f32),
        ),
        emitter: Emitter::Interpolated(
            Coordinates {
                ..Default::default()
            },
            0,
        ),
    };
    let result = scene.chunks::<typenum::U10>();
    assert_eq!(
        (2.02f32, 2.02f32, 2.02f32),
        (result.size_x, result.size_y, result.size_z)
    );
    assert_eq!(
        Coordinates::at(-10.1f32, -10.1f32, -10.1f32),
        result.chunk_starts
    );

    let mut set_chunks: GenericArray<bool, typenum::U1000> = GenericArray::default();
    let mut chunks: HashMap<u32, SceneChunk> = HashMap::new();
    for x in 0..10 {
        for y in 0..10 {
            set_chunks[x * 100 + y * 10] = true;
            set_chunks[x * 100 + y * 10 + 9] = true;
            set_chunks[x * 10 + y] = true;
            set_chunks[x * 10 + y * 1 + 900] = true;
            set_chunks[x * 100 + y] = true;
            set_chunks[x * 100 + y * 1 + 90] = true;
        }
    }
    set_chunks[444] = true;
    set_chunks[544] = true;
    set_chunks[545] = true;
    set_chunks[555] = true;
    assert_set_chunks_equal(&set_chunks, &result);

    chunks.insert(
        444,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![TimedChunkEntry::Dynamic(0, 0, 4)],
        },
    );
    chunks.insert(
        544,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 0, 4),
                TimedChunkEntry::Dynamic(0, 5, 5),
                TimedChunkEntry::Dynamic(0, 6, 8),
                TimedChunkEntry::Dynamic(0, 9, 9),
            ],
        },
    );
    chunks.insert(
        545,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 6, 8),
                TimedChunkEntry::Dynamic(0, 9, 9),
                TimedChunkEntry::Dynamic(0, 10, 11),
                TimedChunkEntry::Dynamic(0, 12, 18),
            ],
        },
    );
    chunks.insert(
        555,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 12, 18),
                TimedChunkEntry::Dynamic(0, 19, 20),
                TimedChunkEntry::Dynamic(0, 21, 29),
            ],
        },
    );

    let mut expected: Chunks<typenum::U10> = Chunks {
        set_chunks,
        chunks,
        size_x: 2.02,
        size_y: 2.02,
        size_z: 2.02,
        chunk_starts: Coordinates::at(-10.1f32, -10.1f32, -10.1f32),
    };

    for x in 0..10 {
        for y in 0..10 {
            // left
            expected.add_surface_at(0, x, y, 0, None);
            expected.add_surface_at(0, x, y, 1, None);
            // front
            expected.add_surface_at(x, 0, y, 2, None);
            expected.add_surface_at(x, 0, y, 3, None);
            // bottom
            expected.add_surface_at(x, y, 0, 8, None);
            expected.add_surface_at(x, y, 0, 9, None);
            // right
            expected.add_surface_at(9, x, y, 4, None);
            expected.add_surface_at(9, x, y, 5, None);
            // back
            expected.add_surface_at(x, 9, y, 6, None);
            expected.add_surface_at(x, 9, y, 7, None);
            // bottom
            expected.add_surface_at(x, y, 9, 10, None);
            expected.add_surface_at(x, y, 9, 11, None);
        }
    }

    assert_chunks_equal(&expected.chunks, &result);

    assert_eq!(expected, result);
}
