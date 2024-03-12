use approx::abs_diff_eq;
use nalgebra::Vector3;

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

#[cfg(test)]
mod tests {
    use crate::maths::is_point_inside_triangle;

    use nalgebra::Vector3;

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
