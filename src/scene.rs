/// Base coordinates struct. Holds x, y and z coordinates.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Coordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Coordinates {
    /// Constructor for coordinates at the given position.
    pub fn at(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z
        }
    }

    /// Add the given value to all three coordinates.
    /// Used for padding maximum bounds.
    /// 
    /// # Example
    /// ```
    /// use demo::scene::Coordinates;
    /// let mut coords = Coordinates::at(1f32, 2f32,3f32);
    /// coords.add(3f32);
    /// assert_eq!(Coordinates::at(4f32, 5f32, 6f32), coords);
    /// ```
    pub fn add(&mut self, value: f32) {
        self.x += value;
        self.y += value;
        self.z += value;
    }

    /// Get the minimum coordinates between these coordinates and `other`.
    /// 
    /// # Example
    /// ```
    /// use demo::scene::Coordinates;
    /// let coords1 = Coordinates::at(1f32, 2f32,3f32);
    /// let coords2 = Coordinates::at(6f32, 2f32, 1f32);
    /// assert_eq!(Coordinates::at(1f32, 2f32, 1f32), coords1.min_coords(&coords2));
    /// ```
    pub fn min_coords(&self, other: &Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    /// Get the maximum coordinates between these coordinates and `other`.
    /// 
    /// # Example
    /// ```
    /// use demo::scene::Coordinates;
    /// let coords1 = Coordinates::at(1f32, 2f32, 3f32);
    /// let coords2 = Coordinates::at(6f32, 2f32, 1f32);
    /// assert_eq!(Coordinates::at(1f32, 2f32, 1f32), coords1.min_coords(&coords2));
    /// ```
    pub fn max_coords(&self, other: &Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }
}

impl Default for Coordinates {
    fn default() -> Self {
        Self {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        }
    }
}

/// Keyframe for a single set of coordinates.
#[derive(Clone, PartialEq, Debug)]
pub struct CoordinateKeyframe {
    pub time: u32,
    pub coords: Coordinates,
}

/// Sound emitter.
/// Either has its separate keyframes (sorted by time) or a single interpolated keyframe at a given time.
#[derive(Clone, PartialEq, Debug)]
pub enum Emitter {
    Keyframes(Vec<CoordinateKeyframe>),
    Interpolated(Coordinates, u32)
}

/// Sound receiver.
/// Either has its separate keyframes (sorted by time) or a single interpolated keyframe at a given time.
/// Always also has a radius.
#[derive(Clone, PartialEq, Debug)]
pub enum Receiver {
    Keyframes(Vec<CoordinateKeyframe>, f32),
    Interpolated(Coordinates, f32, u32)
}

/// Keyframe for a set of coordinates for a surface.
#[derive(Clone, PartialEq, Debug)]
pub struct SurfaceKeyframe<const N: usize> {
    pub time: u32,
    pub coords: [Coordinates; N],
}

/// Surface in the scene.
/// Either has its separate keyframes (sorted by time) or a single interpolated keyframe at a given time.
#[derive(Clone, PartialEq, Debug)]
pub enum Surface<const N: usize> {
    Keyframes(Vec<SurfaceKeyframe<N>>),
    Interpolated([Coordinates; N], u32)
}

/// The full scene.
/// Scenes always have a single emitter and receiver, but support multiple surfaces.
#[derive(Clone, PartialEq, Debug)]
pub struct Scene {
    pub surfaces: Vec<Surface<3>>, // for now we only work with triangles
    pub receiver: Receiver,
    pub emitter: Emitter,
}

#[cfg(test)]
mod tests {
    use super::Coordinates;

    fn test_min_coords() {
        let coords1 = Coordinates::at(2f32, 6f32, 0f32);
        let coords2 = Coordinates::at(5f32, -1f32, 2f32);
        let result = coords1.min_coords(&coords2);
        assert_eq!(Coordinates::at(2f32, -1f32, 0f32), result)
    }

    fn test_max_coords() {
        let coords1 = Coordinates::at(2f32, 6f32, 0f32);
        let coords2 = Coordinates::at(5f32, -1f32, 2f32);
        let result = coords1.min_coords(&coords2);
        assert_eq!(Coordinates::at(5f32, 6f32, 2f32), result)
    }
}
