pub const ABSORPTION_COEFFICIENT_CONCRETE: f64 = 0.98;
pub const MATERIAL_CONCRETE_WALL: Material = Material {
    absorption_coefficient: ABSORPTION_COEFFICIENT_CONCRETE,
    diffusion_coefficient: 0.1f64, // no data for this to be found, so just guess :(
};

/// Data structure representing a material.
/// A material has both an absorption coefficient
/// (denoting how much energy a ray loses when bouncing off of it)
/// and a diffusion coefficient
/// (denoting how diffuse vs. specular the reflection is)
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Material {
    pub absorption_coefficient: f64,
    pub diffusion_coefficient: f64,
}

impl Material {
    /// Randomly choose whether a bounce should be diffuse or not.
    /// A random number between 0 and 1 is rolled and compared to the diffusion coefficient.
    /// If the diffusion coefficient is greater than the random number, the bounce is diffuse.
    pub fn is_bounce_diffuse(&self) -> bool {
        self.diffusion_coefficient >= rand::random::<f64>()
    }
}
