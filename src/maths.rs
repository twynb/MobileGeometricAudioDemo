use approx::abs_diff_eq;
use nalgebra::{Complex, ComplexField, Vector3};
use num::Zero;

trait RealCubeRoot {
    /// This is an altered version of Complex's usual cbrt() implementation.
    /// usually, complex.cbrt() will not provide the real cube root for negative
    /// real numbers and instead gives a complex number.
    /// since we generally want to work with real numbers where possible, this breaks
    /// our system.
    fn real_cbrt(&self) -> Self;
}

impl RealCubeRoot for Complex<f64> {
    fn real_cbrt(&self) -> Self {
        if self.im.is_zero() {
            // simple real ∛r, and copy `im` for its sign
            Self::new(self.re.cbrt(), self.im)
        } else if self.re.is_zero() {
            // ∛(r e^(iπ/2)) = ∛r e^(iπ/6) = ∛r√3/2 + i∛r/2
            // ∛(r e^(-iπ/2)) = ∛r e^(-iπ/6) = ∛r√3/2 - i∛r/2
            let im = self.im.abs().cbrt() / 2f64;
            let re = 3f64.sqrt() * im;
            if self.im.is_sign_positive() {
                Self::new(re, im)
            } else {
                Self::new(re, -im)
            }
        } else {
            // formula: cbrt(r e^(it)) = cbrt(r) e^(it/3)
            let (r, theta) = self.to_polar();
            Self::from_polar(r.cbrt(), theta / 3f64)
        }
    }
}

/// Solve the given cubic equation a3*x^3 + a2*x^2 + a1*x^2 + a0 = 0 and return all real roots.
/// Results with imaginary parts are discarded.
/// This uses the General Cubic Formula as described in Abramowitz/Stegun's Handbook of Mathematical Functions.
///
/// TODO: Optimise by returning a [Option<f64>; 3] rather than a Vec<f64>
pub fn solve_cubic_equation(a3: f64, mut a2: f64, mut a1: f64, mut a0: f64) -> Vec<f64> {
    if a3 == 0f64 {
        return solve_quadratic_equation(a2, a1, a0);
    }
    a2 /= a3;
    a1 /= a3;
    a0 /= a3;

    let q = a1 / 3f64 - a2.powi(2) / 9f64;
    let r = a1.mul_add(a2, -3f64 * a0) / 6f64 - a2.powi(3) / 27f64;

    // readable/non-optimised version
    // let q_r_sqrt: Complex<f64> = Complex::new(q.powi(3) + r.powi(2), 0f64).sqrt();
    let q_r_sqrt: Complex<f64> = Complex::new(r.mul_add(r, q.powi(3)), 0f64).sqrt();

    let s1 = (r + q_r_sqrt).real_cbrt();
    let s2 = (r - q_r_sqrt).real_cbrt();
    let s_added = s1 + s2;
    let s_subbed = s1 - s2;

    let z2_z3_part = Complex::new(0f64, 3f64.sqrt() * 0.5f64) * s_subbed;

    let mut results = Vec::with_capacity(3);
    if s_added.imaginary() == 0f64 {
        results.push(s_added.real() - a2 / 3f64);
    }
    let z2 = -s_added * 0.5f64 - a2 / 3f64 + z2_z3_part;
    let z3 = -s_added * 0.5f64 - a2 / 3f64 - z2_z3_part;
    if z2.imaginary() == 0f64 && !results.contains(&z2.real()) {
        results.push(z2.real());
    }
    if z3.imaginary() == 0f64 && !results.contains(&z3.real()) {
        results.push(z3.real());
    }

    results
}

/// Solve the given quadratic equation a2 * x^2 + a1 * x + a0 = 0 and return all real roots.
/// Results with imaginary parts are discarded.
/// This uses the general quadratic formula as described in Abramowitz/Stegun's Handbook of Mathematical Functions.
///
/// TODO: Optimise by returning a [Option<f64>; 2] rather than a Vec<f64>
pub fn solve_quadratic_equation(a2: f64, a1: f64, a0: f64) -> Vec<f64> {
    if a2 == 0f64 {
        return solve_linear_equation(a1, a0);
    }
    let q = a1.mul_add(a1, -4f64 * a2 * a0);
    if q < 0f64 {
        vec![]
    } else if q == 0f64 {
        vec![-0.5f64 * a1 / a2]
    } else {
        let first_part = -0.5f64 * a1 / a2;
        let q_mul = q.sqrt() * 0.5 / a2;
        vec![first_part + q_mul, first_part - q_mul]
    }
}

/// Solve the given linear equation a1 * x + a0 = 0 and return a vec
/// holding the single result, or nothing if a1 is 0
pub fn solve_linear_equation(a1: f64, a0: f64) -> Vec<f64> {
    if a1 == 0f64 {
        vec![]
    } else {
        vec![-a0 / a1]
    }
}

/// Check whether the given point is within the triangle described by the given vector.
/// This is done by calculating the barycentric coordinates and checking whether they
/// indicate the point is within the triangle.
/// Note that this does not check whether the point is inside the triangle's plane
/// and instead projects it into it!
pub fn is_point_inside_triangle(point: &Vector3<f64>, triangle: &[Vector3<f64>; 3]) -> bool {
    barycentric_coords_inside_triangle(barycentric_coords(point, triangle))
}

/// Get the barycentric coordinates for the given point in the given vector.
/// This will project the point into the same plane as the triangle.
/// based on [this solution from Karadeniz Technical University](https://ceng2.ktu.edu.tr/~cakir/files/grafikler/Texture_Mapping.pdf)
pub fn barycentric_coords(point: &Vector3<f64>, triangle: &[Vector3<f64>; 3]) -> (f64, f64, f64) {
    let v0 = triangle[1] - triangle[0];
    let v1 = triangle[2] - triangle[0];
    let v2 = point - triangle[0];
    let d00 = v0.dot(&v0);
    let d01 = v0.dot(&v1);
    let d11 = v1.dot(&v1);
    let d20 = v2.dot(&v0);
    let d21 = v2.dot(&v1);
    let denom = d00.mul_add(d11, -d01 * d01);
    let beta = d11.mul_add(d20, -d01 * d21) / denom;
    let gamma = d00.mul_add(d21, -d01 * d20) / denom;
    let alpha = 1f64 - beta - gamma;
    (alpha, beta, gamma)
}

/// Check whether the given barycentric coordinates indicate that the described point
/// is within the reference triangle. This is true if all coordinates are >=0 and
/// the three coordinates added up equal 1.
pub fn barycentric_coords_inside_triangle(coords: (f64, f64, f64)) -> bool {
    0f64 <= coords.0
        && 0f64 <= coords.1
        && 0f64 <= coords.2
        && abs_diff_eq!(coords.0 + coords.1 + coords.2, 1f64)
}

/// Truncate to n significant digits, always rounding down.
/// Based on [this solution by stackoverflow user Traumflug](https://stackoverflow.com/a/76572321/16293155)
pub fn trunc_to_n_significant_digits(number: f64, decimals: u32) -> f64 {
    if number == 0. || decimals == 0 {
        return 0.;
    }
    let shift = decimals as i32 - number.abs().log10().ceil() as i32;
    let shift_factor = 10_f64.powi(shift);

    (number * shift_factor).trunc() / shift_factor
}

#[cfg(test)]
mod tests {
    use crate::maths::is_point_inside_triangle;

    use super::{solve_cubic_equation, solve_quadratic_equation};
    use approx::assert_abs_diff_eq;
    use nalgebra::Vector3;

    #[test]
    fn solve_empty_cubic_equation() {
        let vec: Vec<f64> = vec![];
        assert_eq!(vec, solve_cubic_equation(0f64, 0f64, 0f64, 0f64))
    }

    #[test]
    fn solve_basic_cubic_equation() {
        assert_eq!(vec![0f64], solve_cubic_equation(1f64, 0f64, 0f64, 0f64))
    }

    #[test]
    fn solve_basic_cubic_equation_with_offset() {
        assert_eq!(vec![-1f64], solve_cubic_equation(1f64, 0f64, 0f64, 1f64))
    }

    #[test]
    fn solve_complicated_cubic_equation() {
        let result = solve_cubic_equation(2f64, 1f64, -3f64, -0.4f64);
        let expected = vec![1.07f64, -1.44f64, -0.13f64];
        for idx in 0..3 {
            assert_abs_diff_eq!(expected[idx], result[idx], epsilon = 0.01);
        }
    }

    #[test]
    fn solve_empty_quadratic_equation() {
        let result = solve_quadratic_equation(0f64, 0f64, 0f64);
        let expected: Vec<f64> = vec![];
        assert_eq!(expected, result);
    }

    #[test]
    fn solve_basic_quadratic_equation() {
        let result = solve_quadratic_equation(1f64, 0f64, 0f64);
        let expected: Vec<f64> = vec![0f64];
        assert_eq!(expected, result);
    }

    #[test]
    fn solve_basic_quadratic_equation_with_offset() {
        let result = solve_quadratic_equation(-2f64, 0f64, 2f64);
        let expected: Vec<f64> = vec![-1f64, 1f64];
        assert_eq!(expected, result);
    }

    #[test]
    fn solve_complicated_quadratic_equation() {
        let result = solve_quadratic_equation(-0.3f64, 2.4f64, -2.2f64);
        let expected: Vec<f64> = vec![1.06f64, 6.94f64];
        for idx in 0..2 {
            assert_abs_diff_eq!(expected[idx], result[idx], epsilon = 0.01);
        }
    }

    #[test]
    fn origin_is_inside_triangle_around_it() {
        let point = Vector3::new(0f64, 0f64, 0f64);
        let triangle: [Vector3<f64>; 3] = [
            Vector3::new(-1f64, -1f64, 0f64),
            Vector3::new(1f64, -1f64, 0f64),
            Vector3::new(0f64, 1f64, 0f64),
        ];
        assert_eq!(true, is_point_inside_triangle(&point, &triangle))
    }

    #[test]
    fn origin_is_inside_reverse_triangle_around_it() {
        let point = Vector3::new(0f64, 0f64, 0f64);
        let triangle: [Vector3<f64>; 3] = [
            Vector3::new(-1f64, -1f64, 0f64),
            Vector3::new(0f64, 1f64, 0f64),
            Vector3::new(1f64, -1f64, 0f64),
        ];
        assert_eq!(true, is_point_inside_triangle(&point, &triangle))
    }

    #[test]
    fn origin_is_inside_triangle_over_it() {
        let point = Vector3::new(0f64, 0f64, 0f64);
        let triangle: [Vector3<f64>; 3] = [
            Vector3::new(-1f64, -1f64, 1f64),
            Vector3::new(1f64, -1f64, 1f64),
            Vector3::new(0f64, 1f64, 1f64),
        ];
        assert_eq!(true, is_point_inside_triangle(&point, &triangle))
    }

    #[test]
    fn origin_is_outside_triangle_next_to_it() {
        let point = Vector3::new(0f64, 0f64, 0f64);
        let triangle: [Vector3<f64>; 3] = [
            Vector3::new(1f64, -1f64, 0f64),
            Vector3::new(3f64, -1f64, 0f64),
            Vector3::new(1f64, 1f64, 0f64),
        ];
        assert_eq!(false, is_point_inside_triangle(&point, &triangle))
    }

    #[test]
    fn point_next_to_origin_triangle_is_outside() {
        let point = Vector3::new(3f64, 0f64, 0f64);
        let triangle: [Vector3<f64>; 3] = [
            Vector3::new(-1f64, -1f64, 0f64),
            Vector3::new(1f64, -1f64, 0f64),
            Vector3::new(0f64, 1f64, 0f64),
        ];
        assert_eq!(false, is_point_inside_triangle(&point, &triangle))
    }

    #[test]
    fn point_outside_origin_is_inside_triangle_around_it() {
        let point = Vector3::new(2.1f64, -0.3f64, 0f64);
        let triangle: [Vector3<f64>; 3] = [
            Vector3::new(1f64, -1f64, 0f64),
            Vector3::new(3f64, -1f64, 0f64),
            Vector3::new(1f64, 1f64, 0f64),
        ];
        assert_eq!(true, is_point_inside_triangle(&point, &triangle))
    }
}
