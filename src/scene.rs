use nalgebra::Vector3;

use crate::materials::Material;

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
/// Also contains the surface's material.
#[derive(Clone, PartialEq, Debug)]
pub enum Surface<const N: usize> {
    Keyframes(Vec<SurfaceKeyframe<N>>, Material),
    Interpolated([Vector3<f32>; N], u32, Material),
}

impl<const N: usize> Surface<N> {
    /// Calculate this surface's normal.
    /// 
    /// # Panics
    /// 
    /// * When attempting to calculate the normal on a non-interpolated surface.
    pub fn normal(&self) -> Vector3<f32> {
        match self {
            Surface::Interpolated(coords, _time, _material) => {
                let cross = (coords[1] - coords[0]).cross(&(coords[2] - coords[0]));
                cross / cross.norm()
            }
            Surface::Keyframes(_, _material) => {
                panic!("Normals can only be calculated for interpolated surfaces!")
            }
        }
    }
}

/// The full scene.
/// Scenes always have a single emitter and receiver, but support multiple surfaces.
#[derive(Clone, PartialEq, Debug)]
pub struct Scene {
    pub surfaces: Vec<Surface<3>>, // for now we only work with triangles
    pub receiver: Receiver,
    pub emitter: Emitter,
}
