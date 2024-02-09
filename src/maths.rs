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

impl RealCubeRoot for Complex<f32> {
    fn real_cbrt(&self) -> Complex<f32> {
        if self.im.is_zero() {
            // simple real ∛r, and copy `im` for its sign
            Self::new(self.re.cbrt(), self.im)
        } else if self.re.is_zero() {
            // ∛(r e^(iπ/2)) = ∛r e^(iπ/6) = ∛r√3/2 + i∛r/2
            // ∛(r e^(-iπ/2)) = ∛r e^(-iπ/6) = ∛r√3/2 - i∛r/2
            let im = self.im.abs().cbrt() / 2f32;
            let re = 3f32.sqrt() * im;
            if self.im.is_sign_positive() {
                Self::new(re, im)
            } else {
                Self::new(re, -im)
            }
        } else {
            // formula: cbrt(r e^(it)) = cbrt(r) e^(it/3)
            let (r, theta) = self.to_polar();
            Self::from_polar(r.cbrt(), theta / 3f32)
        }
    }
}

/// Solve the given cubic equation a3*x^3 + a2*x^2 + a1*x^2 + a0 = 0 and return all real roots.
/// Results with imaginary parts are discarded.
/// This uses the General Cubic Formula as described in Abramowitz/Stegun's Handbook of Mathematical Functions.
/// 
/// TODO: Optimise by returning a [Option<f32>; 3] rather than a Vec<f32>
pub(crate) fn solve_cubic_equation(a3: f32, mut a2: f32, mut a1: f32, mut a0: f32) -> Vec<f32> {
    if a3 == 0f32 {
        return vec![];
    }
    a2 /= a3;
    a1 /= a3;
    a0 /= a3;

    let q = a1 / 3f32 - a2.powi(2) / 9f32;
    let r = (a1 * a2 - 3f32 * a0) / 6f32 - a2.powi(3) / 27f32;

    let q_r_sqrt: Complex<f32> = Complex::new(q.powi(3) + r.powi(2), 0f32).sqrt();

    let s1 = (r + q_r_sqrt).real_cbrt();
    let s2 = (r - q_r_sqrt).real_cbrt();
    let s_added = s1 + s2;
    let s_subbed = s1 - s2;

    let z2_z3_part = Complex::new(0f32, 3f32.sqrt() * 0.5f32) * s_subbed;

    let mut results = Vec::with_capacity(3);
    if s_added.imaginary() == 0f32 {
        results.push(s_added.real() - a2 / 3f32);
    }
    let z2 = -s_added * 0.5f32 - a2 / 3f32 + z2_z3_part;
    let z3 = -s_added * 0.5f32 - a2 / 3f32 - z2_z3_part;
    if z2.imaginary() == 0f32 && !results.contains(&z2.real()) {
        results.push(z2.real());
    }
    if z3.imaginary() == 0f32 && !results.contains(&z3.real()) {
        results.push(z3.real());
    }

    results
}

/// Solve the given quadratic equation a2 * x^2 + a1 * x + a0 = 0 and return all real roots.
/// Results with imaginary parts are discarded.
/// This uses the general quadratic formula as described in Abramowitz/Stegun's Handbook of Mathematical Functions.
/// 
/// TODO: Optimise by returning a [Option<f32>; 2] rather than a Vec<f32>
pub(crate) fn solve_quadratic_equation(a2: f32, a1: f32, a0: f32) -> Vec<f32> {
    if a2 == 0f32 {
        return vec![];
    }
    let q = a1.powi(2) - 4f32 * a2 * a0;
    if q < 0f32 {
        vec![]
    } else if q == 0f32 {
        vec![-0.5f32 * a1 / a2]
    } else {
        let first_part = -0.5f32 * a1 / a2;
        let q_mul = q.sqrt() * 0.5 / a2;
        vec![first_part + q_mul, first_part - q_mul]
    }
}

/// Check whether the given point is within the triangle described by the given vector.
/// This is done by calculating the barycentric coordinates and checking whether they
/// indicate the point is within the triangle.
pub(crate) fn is_point_inside_triangle(point: &Vector3<f32>, triangle: &[Vector3<f32>; 3]) -> bool {
    barycentric_coords_inside_triangle(barycentric_coords(point, triangle))
}

/// Get the barycentric coordinates for the given point in the given vector.
/// This assumes that the point is within the same plane as the triangle.
pub(crate) fn barycentric_coords(
    point: &Vector3<f32>,
    triangle: &[Vector3<f32>; 3],
) -> (f32, f32, f32) {
    let area = (triangle[1] - triangle[0])
        .cross(&(triangle[2] - triangle[0]))
        .norm();
    let alpha = (triangle[1] - point).cross(&(triangle[2] - point)).norm() / area;
    let beta = (triangle[2] - point).cross(&(triangle[0] - point)).norm() / area;
    let gamma = 1f32 - alpha - beta;
    (alpha, beta, gamma)
}

/// Check whether the given barycentric coordinates indicate that the described point
/// is within the reference triangle. This is true if all coordinates are >=0 and
/// the three coordinates added up equal 1.
pub(crate) fn barycentric_coords_inside_triangle(coords: (f32, f32, f32)) -> bool {
    0f32 <= coords.0
        && 0f32 <= coords.1
        && 0f32 <= coords.2
        && abs_diff_eq!(coords.0 + coords.1 + coords.2, 1f32)
}

#[cfg(test)]
mod tests {
    use crate::maths::is_point_inside_triangle;

    use super::{solve_cubic_equation, solve_quadratic_equation};
    use approx::assert_abs_diff_eq;
    use nalgebra::Vector3;

    #[test]
    fn solve_empty_cubic_equation() {
        let vec: Vec<f32> = vec![];
        assert_eq!(vec, solve_cubic_equation(0f32, 0f32, 0f32, 0f32))
    }

    #[test]
    fn solve_basic_cubic_equation() {
        assert_eq!(vec![0f32], solve_cubic_equation(1f32, 0f32, 0f32, 0f32))
    }

    #[test]
    fn solve_basic_cubic_equation_with_offset() {
        assert_eq!(vec![-1f32], solve_cubic_equation(1f32, 0f32, 0f32, 1f32))
    }

    #[test]
    fn solve_complicated_cubic_equation() {
        let result = solve_cubic_equation(2f32, 1f32, -3f32, -0.4f32);
        let expected = vec![1.07f32, -1.44f32, -0.13f32];
        for idx in 0..3 {
            assert_abs_diff_eq!(expected[idx], result[idx], epsilon = 0.01);
        }
    }

    #[test]
    fn solve_empty_quadratic_equation() {
        let result = solve_quadratic_equation(0f32, 0f32, 0f32);
        let expected: Vec<f32> = vec![];
        assert_eq!(expected, result);
    }

    #[test]
    fn solve_basic_quadratic_equation() {
        let result = solve_quadratic_equation(1f32, 0f32, 0f32);
        let expected: Vec<f32> = vec![0f32];
        assert_eq!(expected, result);
    }

    #[test]
    fn solve_basic_quadratic_equation_with_offset() {
        let result = solve_quadratic_equation(-2f32, 0f32, 2f32);
        let expected: Vec<f32> = vec![-1f32, 1f32];
        assert_eq!(expected, result);
    }

    #[test]
    fn solve_complicated_quadratic_equation() {
        let result = solve_quadratic_equation(-0.3f32, 2.4f32, -2.2f32);
        let expected: Vec<f32> = vec![1.06f32, 6.94f32];
        for idx in 0..2 {
            assert_abs_diff_eq!(expected[idx], result[idx], epsilon = 0.01);
        }
    }

    #[test]
    fn origin_is_inside_triangle_around_it() {
        let point = Vector3::new(0f32, 0f32, 0f32);
        let triangle: [Vector3<f32>; 3] = [
            Vector3::new(-1f32, -1f32, 0f32),
            Vector3::new(1f32, -1f32, 0f32),
            Vector3::new(0f32, 1f32, 0f32),
        ];
        assert_eq!(true, is_point_inside_triangle(&point, &triangle))
    }

    #[test]
    fn origin_is_inside_reverse_triangle_around_it() {
        let point = Vector3::new(0f32, 0f32, 0f32);
        let triangle: [Vector3<f32>; 3] = [
            Vector3::new(-1f32, -1f32, 0f32),
            Vector3::new(0f32, 1f32, 0f32),
            Vector3::new(1f32, -1f32, 0f32),
        ];
        assert_eq!(true, is_point_inside_triangle(&point, &triangle))
    }


    #[test]
    fn origin_is_outside_triangle_over_it() {
        let point = Vector3::new(0f32, 0f32, 0f32);
        let triangle: [Vector3<f32>; 3] = [
            Vector3::new(-1f32, -1f32, 1f32),
            Vector3::new(1f32, -1f32, 1f32),
            Vector3::new(0f32, 1f32, 1f32),
        ];
        assert_eq!(false, is_point_inside_triangle(&point, &triangle))
    }

    #[test]
    fn origin_is_outside_triangle_next_to_it() {
        let point = Vector3::new(0f32, 0f32, 0f32);
        let triangle: [Vector3<f32>; 3] = [
            Vector3::new(1f32, -1f32, 0f32),
            Vector3::new(3f32, -1f32, 0f32),
            Vector3::new(1f32, 1f32, 0f32),
        ];
        assert_eq!(false, is_point_inside_triangle(&point, &triangle))
    }

    #[test]
    fn point_next_to_origin_triangle_is_outside() {
        let point = Vector3::new(3f32, 0f32, 0f32);
        let triangle: [Vector3<f32>; 3] = [
            Vector3::new(-1f32, -1f32, 0f32),
            Vector3::new(1f32, -1f32, 0f32),
            Vector3::new(0f32, 1f32, 0f32),
        ];
        assert_eq!(false, is_point_inside_triangle(&point, &triangle))
    }

    #[test]
    fn point_outside_origin_is_inside_triangle_around_it() {
        let point = Vector3::new(2.1f32, 0.3f32, 0f32);
        let triangle: [Vector3<f32>; 3] = [
            Vector3::new(1f32, -1f32, 0f32),
            Vector3::new(3f32, -1f32, 0f32),
            Vector3::new(1f32, 1f32, 0f32),
        ];
        assert_eq!(true, is_point_inside_triangle(&point, &triangle))
    }
}
