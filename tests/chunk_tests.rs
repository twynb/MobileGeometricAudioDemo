use std::collections::HashMap;

use generic_array::GenericArray;

use demo::{
    bounce::EmissionType,
    chunk::{Chunks, SceneChunk, TimedChunkEntry},
    materials::MATERIAL_CONCRETE_WALL,
    scene::{CoordinateKeyframe, Emitter, Receiver, Scene, Surface, SurfaceKeyframe},
    scene_builder,
};
use itertools::Itertools;
use nalgebra::Vector3;

fn empty_scene() -> Scene {
    Scene {
        receiver: Receiver::Interpolated(Vector3::new(0f64, 0f64, 0f64), 0.1, 0),
        surfaces: vec![],
        emitter: Emitter::Keyframes(
            vec![CoordinateKeyframe {
                time: 0,
                coords: Vector3::new(0f64, 0f64, 0f64),
            }],
            EmissionType::Random,
        ),
        loop_duration: None,
    }
}

fn static_scene_moving_receiver() -> Scene {
    Scene {
        receiver: Receiver::Keyframes(
            vec![
                CoordinateKeyframe {
                    time: 10,
                    coords: Vector3::new(-1f64, -1f64, -1f64),
                },
                CoordinateKeyframe {
                    time: 20,
                    coords: Vector3::new(1f64, -1f64, 0f64),
                },
                CoordinateKeyframe {
                    time: 40,
                    coords: Vector3::new(1f64, 1f64, 1f64),
                },
            ],
            0.1,
        ),
        surfaces: scene_builder::static_cube(
            Vector3::new(-10f64, -10f64, -10f64),
            Vector3::new(10f64, 10f64, 10f64),
            MATERIAL_CONCRETE_WALL,
        ),
        emitter: Emitter::Interpolated(Vector3::new(0f64, 0f64, 0f64), 0, EmissionType::Random),
        loop_duration: None,
    }
}

fn moving_scene_and_receiver() -> Scene {
    let mut surfaces = scene_builder::static_cube(
        Vector3::new(-10f64, -10f64, -10f64),
        Vector3::new(10f64, 10f64, 10f64),
        MATERIAL_CONCRETE_WALL,
    );
    surfaces.push(Surface::Keyframes(
        vec![
            SurfaceKeyframe {
                time: 20,
                coords: [
                    Vector3::new(2f64, 2f64, 2f64),
                    Vector3::new(2f64, 2f64, 3f64),
                    Vector3::new(2f64, 3f64, 2f64),
                ],
            },
            SurfaceKeyframe {
                time: 500,
                coords: [
                    Vector3::new(6f64, 6f64, 6f64),
                    Vector3::new(6f64, 6f64, 7f64),
                    Vector3::new(6f64, 7f64, 6f64),
                ],
            },
        ],
        MATERIAL_CONCRETE_WALL,
    ));
    Scene {
        receiver: Receiver::Keyframes(
            vec![
                CoordinateKeyframe {
                    time: 10,
                    coords: Vector3::new(-1f64, -1f64, -1f64),
                },
                CoordinateKeyframe {
                    time: 20,
                    coords: Vector3::new(1f64, -1f64, 0f64),
                },
                CoordinateKeyframe {
                    time: 40,
                    coords: Vector3::new(1f64, 1f64, 1f64),
                },
            ],
            0.1,
        ),
        surfaces,
        emitter: Emitter::Interpolated(Vector3::new(0f64, 0f64, 0f64), 0, EmissionType::Random),
        loop_duration: None,
    }
}

fn looping_moving_scene_and_receiver() -> Scene {
    let mut surfaces = scene_builder::static_cube(
        Vector3::new(-10f64, -10f64, -10f64),
        Vector3::new(10f64, 10f64, 10f64),
        MATERIAL_CONCRETE_WALL,
    );
    surfaces.push(Surface::Keyframes(
        vec![
            SurfaceKeyframe {
                time: 0,
                coords: [
                    Vector3::new(2f64, 2f64, 2f64),
                    Vector3::new(2f64, 2f64, 3f64),
                    Vector3::new(2f64, 3f64, 2f64),
                ],
            },
            SurfaceKeyframe {
                time: 480,
                coords: [
                    Vector3::new(6f64, 6f64, 6f64),
                    Vector3::new(6f64, 6f64, 7f64),
                    Vector3::new(6f64, 7f64, 6f64),
                ],
            },
        ],
        MATERIAL_CONCRETE_WALL,
    ));
    Scene {
        receiver: Receiver::Keyframes(
            vec![
                CoordinateKeyframe {
                    time: 10,
                    coords: Vector3::new(-1f64, -1f64, -1f64),
                },
                CoordinateKeyframe {
                    time: 20,
                    coords: Vector3::new(1f64, -1f64, 0f64),
                },
                CoordinateKeyframe {
                    time: 40,
                    coords: Vector3::new(1f64, 1f64, 1f64),
                },
                CoordinateKeyframe {
                    time: 480,
                    coords: Vector3::new(1f64, 1f64, 1f64),
                },
            ],
            0.1,
        ),
        surfaces,
        emitter: Emitter::Interpolated(Vector3::new(0f64, 0f64, 0f64), 0, EmissionType::Random),
        loop_duration: Some(480),
    }
}

#[allow(clippy::all)]
fn assert_set_chunks_equal(
    set_chunks: &GenericArray<bool, typenum::U1000>,
    result: &Chunks<typenum::U10>,
) {
    for idx in 0..1000 {
        assert_eq!(
            set_chunks[idx], result.set_chunks[idx],
            "mismatch in set_chunks at index {idx}",
        );
    }
}

#[allow(clippy::all)]
fn assert_chunks_equal(chunks: &HashMap<u32, SceneChunk>, result: &Chunks<typenum::U10>) {
    for idx in 0..1000 {
        assert_eq!(
            chunks.get(&idx),
            result.chunks.get(&idx),
            "mismatch in chunks at index {idx}",
        );
    }
}

#[allow(clippy::all)]
#[test]
fn chunks_empty_scene() {
    let scene = empty_scene();
    let result = scene.chunks::<typenum::U10>();
    assert_eq!(
        (0.04f64, 0.04f64, 0.04f64),
        (result.size_x, result.size_y, result.size_z)
    );
    assert_eq!(Vector3::new(-0.2f64, -0.2f64, -0.2f64), result.chunk_starts);

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
        chunk_starts: Vector3::new(-0.2f64, -0.2f64, -0.2f64),
    };
    assert_eq!(expected, result);
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::all)]
#[test]
fn chunks_static_scene_moving_receiver() {
    let scene = static_scene_moving_receiver();
    let result = scene.chunks::<typenum::U10>();
    assert_eq!(
        (2.02f64, 2.02f64, 2.02f64),
        (result.size_x, result.size_y, result.size_z)
    );
    assert_eq!(
        Vector3::new(-10.1f64, -10.1f64, -10.1f64),
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
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 0, 10),
                TimedChunkEntry::Dynamic(0, 10, 14),
                TimedChunkEntry::Dynamic(0, 15, 15),
            ],
        },
    );
    chunks.insert(
        544,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 15, 15),
                TimedChunkEntry::Dynamic(0, 16, 18),
                TimedChunkEntry::Dynamic(0, 19, 19),
                TimedChunkEntry::Dynamic(0, 20, 21),
            ],
        },
    );
    chunks.insert(
        545,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 19, 19),
                TimedChunkEntry::Dynamic(0, 20, 21),
                TimedChunkEntry::Dynamic(0, 22, 28),
                TimedChunkEntry::Dynamic(0, 29, 30),
            ],
        },
    );
    chunks.insert(
        555,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 29, 30),
                TimedChunkEntry::Dynamic(0, 31, 39),
                TimedChunkEntry::Final(0, 40),
            ],
        },
    );

    let mut expected: Chunks<typenum::U10> = Chunks {
        set_chunks,
        chunks,
        size_x: 2.02,
        size_y: 2.02,
        size_z: 2.02,
        chunk_starts: Vector3::new(-10.1f64, -10.1f64, -10.1f64),
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

#[allow(clippy::too_many_lines)]
#[allow(clippy::all)]
#[test]
fn chunks_moving_scene_and_receiver() {
    let scene = moving_scene_and_receiver();
    let result = scene.chunks::<typenum::U10>();
    assert_eq!(
        (2.02f64, 2.02f64, 2.02f64),
        (result.size_x, result.size_y, result.size_z)
    );
    assert_eq!(
        Vector3::new(-10.1f64, -10.1f64, -10.1f64),
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

    set_chunks[565] = true;
    set_chunks[556] = true;
    set_chunks[566] = true;
    set_chunks[666] = true;
    set_chunks[676] = true;
    set_chunks[667] = true;
    set_chunks[677] = true;
    set_chunks[777] = true;
    set_chunks[787] = true;
    set_chunks[778] = true;
    set_chunks[788] = true;
    assert_set_chunks_equal(&set_chunks, &result);

    chunks.insert(
        444,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 0, 10),
                TimedChunkEntry::Dynamic(0, 10, 14),
                TimedChunkEntry::Dynamic(0, 15, 15),
            ],
        },
    );
    chunks.insert(
        544,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 15, 15),
                TimedChunkEntry::Dynamic(0, 16, 18),
                TimedChunkEntry::Dynamic(0, 19, 19),
                TimedChunkEntry::Dynamic(0, 20, 21),
            ],
        },
    );
    chunks.insert(
        545,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 19, 19),
                TimedChunkEntry::Dynamic(0, 20, 21),
                TimedChunkEntry::Dynamic(0, 22, 28),
                TimedChunkEntry::Dynamic(0, 29, 30),
            ],
        },
    );
    chunks.insert(
        555,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 29, 30),
                TimedChunkEntry::Dynamic(0, 31, 39),
                TimedChunkEntry::Final(0, 40),
            ],
        },
    );

    let mut expected: Chunks<typenum::U10> = Chunks {
        set_chunks,
        chunks,
        size_x: 2.02,
        size_y: 2.02,
        size_z: 2.02,
        chunk_starts: Vector3::new(-10.1f64, -10.1f64, -10.1f64),
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
    for y in 5..=6 {
        for z in 5..=6 {
            expected.add_surface_at(5, y, z, 12, Some((0, Some(20))));
            expected.add_surface_at(5, y, z, 12, Some((20, Some(22))));
        }
    }
    expected.add_surface_at(6, 6, 6, 12, Some((23, Some(144))));
    for y in 6..=7 {
        for z in 6..=7 {
            expected.add_surface_at(6, y, z, 12, Some((145, Some(264))));
        }
    }
    expected.add_surface_at(7, 7, 7, 12, Some((265, Some(387))));
    for y in 7..=8 {
        for z in 7..=8 {
            expected.add_surface_at(7, y, z, 12, Some((388, Some(499))));
            expected.add_surface_at(7, y, z, 12, Some((500, None)));
        }
    }

    assert_chunks_equal(&expected.chunks, &result);

    assert_eq!(expected, result);
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::all)]
#[test]
fn chunks_looping_moving_scene_and_receiver() {
    let scene = looping_moving_scene_and_receiver();
    let result = scene.chunks::<typenum::U10>();
    assert_eq!(
        (2.02f64, 2.02f64, 2.02f64),
        (result.size_x, result.size_y, result.size_z)
    );
    assert_eq!(
        Vector3::new(-10.1f64, -10.1f64, -10.1f64),
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

    set_chunks[565] = true;
    set_chunks[556] = true;
    set_chunks[566] = true;
    set_chunks[666] = true;
    set_chunks[676] = true;
    set_chunks[667] = true;
    set_chunks[677] = true;
    set_chunks[777] = true;
    set_chunks[787] = true;
    set_chunks[778] = true;
    set_chunks[788] = true;
    assert_set_chunks_equal(&set_chunks, &result);

    chunks.insert(
        444,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 0, 10),
                TimedChunkEntry::Dynamic(0, 10, 14),
                TimedChunkEntry::Dynamic(0, 15, 15),
            ],
        },
    );
    chunks.insert(
        544,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 15, 15),
                TimedChunkEntry::Dynamic(0, 16, 18),
                TimedChunkEntry::Dynamic(0, 19, 19),
                TimedChunkEntry::Dynamic(0, 20, 21),
            ],
        },
    );
    chunks.insert(
        545,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 19, 19),
                TimedChunkEntry::Dynamic(0, 20, 21),
                TimedChunkEntry::Dynamic(0, 22, 28),
                TimedChunkEntry::Dynamic(0, 29, 30),
            ],
        },
    );
    chunks.insert(
        555,
        SceneChunk {
            surfaces: vec![],
            receivers: vec![
                TimedChunkEntry::Dynamic(0, 29, 30),
                TimedChunkEntry::Dynamic(0, 31, 39),
                TimedChunkEntry::Dynamic(0, 40, 479),
                TimedChunkEntry::Dynamic(0, 480, 480),
            ],
        },
    );

    let mut expected: Chunks<typenum::U10> = Chunks {
        set_chunks,
        chunks,
        size_x: 2.02,
        size_y: 2.02,
        size_z: 2.02,
        chunk_starts: Vector3::new(-10.1f64, -10.1f64, -10.1f64),
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
    for y in 5..=6 {
        for z in 5..=6 {
            expected.add_surface_at(5, y, z, 12, Some((0, Some(2))));
        }
    }
    expected.add_surface_at(6, 6, 6, 12, Some((3, Some(124))));
    for y in 6..=7 {
        for z in 6..=7 {
            expected.add_surface_at(6, y, z, 12, Some((125, Some(244))));
        }
    }
    expected.add_surface_at(7, 7, 7, 12, Some((245, Some(367))));
    for y in 7..=8 {
        for z in 7..=8 {
            expected.add_surface_at(7, y, z, 12, Some((368, Some(479))));
            expected.add_surface_at(7, y, z, 12, Some((480, Some(480))));
        }
    }

    assert_chunks_equal(&expected.chunks, &result);

    assert_eq!(expected, result);
}

#[test]
fn objects_at_loop_start_and_end_match_loop_overarching_times() {
    let scene = looping_moving_scene_and_receiver();
    let chunks = scene.chunks::<typenum::U11>();

    for key in 0..1110 {
        let (mut expected_receivers, mut expected_surfaces) =
            chunks.objects_at_key_and_time(key, 460, 480, None);
        let (expected_receivers_beginning, expected_surfaces_beginning) =
            chunks.objects_at_key_and_time(key, 0, 40, None);
        expected_receivers.extend_from_slice(&expected_receivers_beginning);
        expected_surfaces.extend_from_slice(&expected_surfaces_beginning);
        expected_receivers = expected_receivers.iter().unique().map(|val| *val).collect();
        expected_surfaces = expected_surfaces.iter().unique().map(|val| *val).collect();

        let result = chunks.objects_at_key_and_time(key, 460, 520, Some(480));
        assert_eq!(
            (expected_receivers, expected_surfaces),
            result,
            "Failed for key {key}"
        )
    }
}

#[test]
fn objects_at_loop_covering_span_is_all_objects() {
    let scene = looping_moving_scene_and_receiver();
    let chunks = scene.chunks::<typenum::U11>();
    for key in 0..1110 {
        let expected = chunks.objects_at_key_and_time(key, 0, 1000, None);

        let result = chunks.objects_at_key_and_time(key, 300, 781, Some(480));
        assert_eq!(expected, result, "Failed for key {key}")
    }
}

#[test]
fn objects_at_looping_times_match() {
    let scene = looping_moving_scene_and_receiver();
    let chunks = scene.chunks::<typenum::U11>();

    let mut indices_to_check = Vec::with_capacity(11usize.pow(3));
    for x in 0..11 {
        for y in 0..11 {
            for z in 0..11 {
                indices_to_check.push(x * 11 * 11 + y * 11 + z);
            }
        }
    }

    for a_time in 0..4 {
        for b_time in 1..10 {
            if a_time == b_time {
                continue;
            }
            for index in &indices_to_check {
                assert_eq!(
                    chunks.objects_at_key_and_time(
                        *index,
                        1 + 480 * a_time,
                        100 + 480 * a_time,
                        Some(480)
                    ),
                    chunks.objects_at_key_and_time(
                        *index,
                        1 + 480 * b_time,
                        100 + 480 * b_time,
                        Some(480)
                    )
                );
            }
        }
    }
}
