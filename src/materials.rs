pub const ABSORPTION_COEFFICIENT_CONCRETE: f32 = 0.98;
pub const MATERIAL_CONCRETE_WALL: Material = Material {
    absorption_coefficient: ABSORPTION_COEFFICIENT_CONCRETE,
    diffusion_coefficient: 0.1f32 // no data for this to be found, so just guess :(
};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Material {
    pub absorption_coefficient: f32,
    pub diffusion_coefficient: f32,
}

impl Material {
    /// Randomly choose whether a bounce should be diffuse or not.
    pub fn is_bounce_diffuse(&self) -> bool {
        self.diffusion_coefficient >= rand::random::<f32>()
    }
}