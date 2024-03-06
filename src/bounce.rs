use nalgebra::Vector3;
use rand::random;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum EmissionType {
    Random,
    Directed(Vector3<f64>),
}

impl EmissionType {
    /// Get the direction to bounce from, depending on this emission type.
    /// The returned value is *NOT* guaranteed to be a unit vector!
    pub fn get_direction(&self) -> Vector3<f64> {
        match self {
            Self::Random => random_direction(),
            Self::Directed(dir) => *dir,
        }
    }
}

/// Get a `Vector3` pointing in a random direction.
/// The returned value is *NOT* guaranteed to be a unit vector!
pub fn random_direction() -> Vector3<f64> {
    Vector3::new(
        random::<f64>().mul_add(2f64, -1f64),
        random::<f64>().mul_add(2f64, -1f64),
        random::<f64>().mul_add(2f64, -1f64),
    )
}

/// Get a `Vector3` pointing in a random direction.
/// The returned value is guaranteed to be a unit vector.
pub fn random_unit_direction() -> Vector3<f64> {
    let mut res = random_direction();
    res.normalize_mut();
    res
}

/// Get a `Vector3` pointing in a random direction inside the hemisphere
/// where the given `normal` is the vec from the center to the tip.
/// 
/// To avoid errors, this will avoid overly flat angles.
/// The returned value is guaranteed to be a unit vector.
pub fn random_direction_in_hemisphere(normal: &Vector3<f64>) -> Vector3<f64> {
    let mut result = random_unit_direction();
    while result.dot(normal) <= 0.05f64 {
        result = random_unit_direction();
    }
    result
}

/// Bounce the direction vector off a surface described by the given normal.
/// Assumes that both the direction and normal are unit vectors.
#[allow(clippy::module_name_repetitions)]
pub fn bounce_off_surface_with_normal(direction: &mut Vector3<f64>, normal: &Vector3<f64>) {
    *direction -= 2f64 * (direction.dot(normal)) * normal;
}
