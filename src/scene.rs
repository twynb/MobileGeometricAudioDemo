/// base coordinates pub struct
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Coordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Coordinates {
    pub fn min_coords(&self, other: &Self) -> Self {
        Self {
            x: (self.x / self.w).min(other.x / other.w),
            y: (self.y / self.w).min(other.y / other.w),
            z: (self.z / self.w).min(other.z / other.w),
            w: 1f32,
        }
    }
    
    pub fn max_coords(&self, other: &Self) -> Self {
        Self {
            x: (self.x / self.w).max(other.x / other.w),
            y: (self.y / self.w).max(other.y / other.w),
            z: (self.z / self.w).max(other.z / other.w),
            w: 1f32,
        }
    }
}

impl Default for Coordinates {
    fn default() -> Self {
        Self {
            x: 0f32,
            y: 0f32,
            z: 0f32,
            w: 1f32,
        }
    }
}

/// Keyframe for a single set of coordinates.
#[derive(PartialEq, Debug)]
pub struct CoordinateKeyframe {
    pub time: u32,
    pub coords: Coordinates,
}

/// Sound emitter.
/// `coordinates` should only be Some when this Emitter is returned by the atTime() function TODO
/// `keyframes` is expected to be sorted by keyframe time.
#[derive(PartialEq, Debug)]
pub struct Emitter {
    pub keyframes: Option<Vec<CoordinateKeyframe>>,
    pub index: usize,
    pub coordinates: Option<Coordinates>,
}

/// Sound receiver.
/// `coordinates` should only be Some when this Receiver is returned by the atTime() function TODO
/// `keyframes` is expected to be sorted by keyframe time.
#[derive(PartialEq, Debug)]
pub struct Receiver {
    pub keyframes: Option<Vec<CoordinateKeyframe>>,
    pub index: usize,
    pub coordinates: Option<Coordinates>,
}

/// Keyframe for a set of coordinates for an object.
#[derive(PartialEq, Debug)]
pub struct SurfaceKeyframe<const N: usize> {
    pub time: u32,
    pub coords: [Coordinates; N],
}
/// Object in the scene.
/// `coordinates` should only be Some when this Object is returned by the atTime() function TODO
/// `keyframes` is expected to be sorted by keyframe time.
/// It is expected that all CoordinateKeyframes have the same amount of coordinates.
#[derive(PartialEq, Debug)]
pub struct Surface<const N: usize> {
    pub keyframes: Option<Vec<SurfaceKeyframe<N>>>,
    pub index: usize,
    pub coordinates: Option<[Coordinates; N]>,
}

#[derive(PartialEq, Debug)]
pub struct Scene {
    pub surfaces: Vec<Surface<4>>, // for now we only work with rectangles
    pub receiver: Receiver,
    pub emitter: Emitter,
}
