/// base coordinates pub struct
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Coordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

impl Default for Coordinates {
    fn default() -> Self {
        return Self {
            x: 0f32,
            y: 0f32,
            z: 0f32,
            w: 1f32
        }
    }
}

/// Keyframe for a single set of coordinates.
#[derive(PartialEq, Debug)]
pub struct CoordinateKeyframe {
    pub time: u32,
    pub coords: Coordinates
}

/// Sound emitter.
/// `coordinates` should only be Some when this Emitter is returned by the atTime() function TODO
/// `keyframes` is expected to be sorted by keyframe time.
#[derive(PartialEq, Debug)]
pub struct Emitter {
    pub keyframes: Option<Vec<CoordinateKeyframe>>,
    pub index: usize,
    pub coordinates: Option<Coordinates>
}

/// Sound receiver.
/// `coordinates` should only be Some when this Receiver is returned by the atTime() function TODO
/// `keyframes` is expected to be sorted by keyframe time.
#[derive(PartialEq, Debug)]
pub struct Receiver {
    pub keyframes: Option<Vec<CoordinateKeyframe>>,
    pub index: usize,
    pub coordinates: Option<Coordinates>
}

/// Keyframe for a set of coordinates for an object.
#[derive(PartialEq, Debug)]
pub struct ObjectKeyframe {
    pub time: u32,
    pub coords: Vec<Coordinates>
}

/// Object in the scene.
/// `coordinates` should only be Some when this Object is returned by the atTime() function TODO
/// `keyframes` is expected to be sorted by keyframe time.
/// It is expected that all CoordinateKeyframes have the same amount of coordinates.
#[derive(PartialEq, Debug)]
pub struct Object {
    pub keyframes: Option<Vec<ObjectKeyframe>>,
    pub index: usize,
    pub coordinates: Option<Vec<Coordinates>>
}

#[derive(PartialEq, Debug)]
pub struct Scene {
    pub objects: Vec<Object>,
    pub receiver: Receiver,
    pub emitters: Vec<Emitter>
}