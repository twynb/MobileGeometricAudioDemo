use nalgebra::Vector3;

/// Keyframe for a single set of coordinates.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct CoordinateKeyframe {
    pub time: u32,
    pub coords: Vector3<f32>,
}

/// Sound emitter.
/// Either has its separate keyframes (sorted by time) or a single interpolated keyframe at a given time.
#[derive(Clone, PartialEq, Debug)]
pub enum Emitter {
    Keyframes(Vec<CoordinateKeyframe>),
    Interpolated(Vector3<f32>, u32),
}

/// Sound receiver.
/// Either has its separate keyframes (sorted by time) or a single interpolated keyframe at a given time.
/// Always also has a radius.
#[derive(Clone, PartialEq, Debug)]
pub enum Receiver {
    Keyframes(Vec<CoordinateKeyframe>, f32),
    Interpolated(Vector3<f32>, f32, u32),
}

/// Keyframe for a set of coordinates for a surface.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SurfaceKeyframe<const N: usize> {
    pub time: u32,
    pub coords: [Vector3<f32>; N],
}

/// Surface in the scene.
/// Either has its separate keyframes (sorted by time) or a single interpolated keyframe at a given time.
#[derive(Clone, PartialEq, Debug)]
pub enum Surface<const N: usize> {
    Keyframes(Vec<SurfaceKeyframe<N>>),
    Interpolated([Vector3<f32>; N], u32),
}

/// The full scene.
/// Scenes always have a single emitter and receiver, but support multiple surfaces.
#[derive(Clone, PartialEq, Debug)]
pub struct Scene {
    pub surfaces: Vec<Surface<3>>, // for now we only work with triangles
    pub receiver: Receiver,
    pub emitter: Emitter,
}
