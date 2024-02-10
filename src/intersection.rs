use nalgebra::Vector3;

use crate::interpolation::interpolate_two_surface_keyframes;
use crate::maths;
use crate::scene::CoordinateKeyframe;
use crate::{
    ray::Ray,
    scene::{Receiver, Surface, SurfaceKeyframe},
};

/// Find the first intersection between the given ray and surface.
/// The intersection logic for interpolated/keyframe surfaces is defined in
/// `intersection_check_surface_coordinates` and `intersection_check_surface_keyframes`
/// respectively.
/// For interpolated surfaces, only one check is required because they don't change. For keyframe
/// surfaces, a check between every set of keyframes relevant to the entry/exit time is done.
pub fn intersect_ray_and_surface(
    ray: &Ray,
    surface: &Surface<3>,
    time_entry: u32,
    time_exit: u32,
) -> Option<(u32, Vector3<f32>)> {
    match surface {
        Surface::Interpolated(coords, _time) => {
            intersection_check_surface_coordinates(ray, coords, time_entry, time_exit)
        }
        Surface::Keyframes(keyframes) => {
            for pair in keyframes.windows(2) {
                if pair[1].time < time_entry {
                    continue;
                }
                if pair[0].time > time_exit {
                    return None;
                }
                if let Some((time, coords)) =
                    intersection_check_surface_keyframes(ray, &pair[0], &pair[1])
                {
                    return Some((time, coords));
                }
            }
            // do final check after last keyframe
            let final_keyframe = &keyframes[keyframes.len() - 1];
            intersection_check_surface_coordinates(
                ray,
                &final_keyframe.coords,
                final_keyframe.time,
                time_exit,
            )
        }
    }
}

/// Check for an intersection inbetween the two given keyframes.
/// This uses the logic explained in the "Intersection Checks" chapter of the thesis,
/// with its corresponding variable names.
fn intersection_check_surface_keyframes(
    ray: &Ray,
    keyframe_first: &SurfaceKeyframe<3>,
    keyframe_second: &SurfaceKeyframe<3>,
) -> Option<(u32, Vector3<f32>)> {
    let (d3, d2, d1, d0) = surface_polynomial_parameters(ray, keyframe_first, keyframe_second);

    let intersections = maths::solve_cubic_equation(d3, d2, d1, d0);
    let mut intersection: Option<f32> = None;
    for intersection_time in intersections {
        if (intersection_time.floor() as u32) < keyframe_first.time
            || intersection_time.ceil() as u32 > keyframe_second.time
        {
            continue;
        }
        if match intersection {
            Some(time) => time > intersection_time,
            None => true,
        } {
            intersection = Some(intersection_time);
        }
    }

    let Some(intersection_time) = intersection else {
        return None;
    };

    let Some(surface_coords) =
        interpolate_two_surface_keyframes(keyframe_first, keyframe_second, intersection_time)
    else {
        return None;
    };

    let ray_coords = ray.coords_at_time(intersection_time);

    if maths::is_point_inside_triangle(&ray_coords, &surface_coords) {
        Some((intersection_time.round() as u32, ray_coords))
    } else {
        None
    }
}

/// Calculate the surface intersection polynomial parameters (called d_0 through d_3 in the thesis).
fn surface_polynomial_parameters(
    ray: &Ray,
    keyframe_first: &SurfaceKeyframe<3>,
    keyframe_second: &SurfaceKeyframe<3>,
) -> (f32, f32, f32, f32) {
    let (g2, g1, g0) = surface_cross_product_parameters(keyframe_first, keyframe_second);
    let velocity = ray.velocity * ray.direction.into_inner();
    let delta_time = (keyframe_second.time - keyframe_first.time) as f32;
    let diff_point_1 = keyframe_first.coords[0] - keyframe_second.coords[0];
    let second_div_delta_time = keyframe_second.time as f32 / delta_time;
    let g2_dot_diff_point_1 = g2.dot(&diff_point_1);
    let g1_dot_diff_point_1 = g1.dot(&diff_point_1);
    let g0_dot_diff_point_1 = g0.dot(&diff_point_1);
    (
        g2.dot(&velocity) - g2_dot_diff_point_1 / delta_time, // d_3
        g2.dot(&ray.origin) - g2_dot_diff_point_1
            + g2_dot_diff_point_1 * (&second_div_delta_time)
            + g1.dot(&velocity)
            - g1_dot_diff_point_1 / delta_time, // d_2
        g1.dot(&ray.origin) - g1_dot_diff_point_1
            + g1_dot_diff_point_1 * (&second_div_delta_time)
            + g0.dot(&velocity)
            - g0_dot_diff_point_1 / delta_time, // d_1
        g0.dot(&ray.origin) - g0_dot_diff_point_1 + g0_dot_diff_point_1 * (&second_div_delta_time), // d_0
    )
}

/// Calculate the cross product parameters (called g_0 through g_2 in the thesis).
fn surface_cross_product_parameters(
    keyframe_first: &SurfaceKeyframe<3>,
    keyframe_second: &SurfaceKeyframe<3>,
) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
    let second_time = keyframe_second.time as f32;
    let delta_time = (keyframe_second.time - keyframe_first.time) as f32;
    let two_three = surface_sub_cross_product_parameters(
        &keyframe_first.coords[1],
        &keyframe_second.coords[1],
        &keyframe_first.coords[2],
        &keyframe_second.coords[2],
        delta_time,
        second_time,
    );
    let two_one = surface_sub_cross_product_parameters(
        &keyframe_first.coords[1],
        &keyframe_second.coords[1],
        &keyframe_first.coords[0],
        &keyframe_second.coords[0],
        delta_time,
        second_time,
    );
    let one_three = surface_sub_cross_product_parameters(
        &keyframe_first.coords[0],
        &keyframe_second.coords[0],
        &keyframe_first.coords[2],
        &keyframe_second.coords[2],
        delta_time,
        second_time,
    );
    let one_one = surface_sub_cross_product_parameters(
        &keyframe_first.coords[0],
        &keyframe_second.coords[0],
        &keyframe_first.coords[0],
        &keyframe_second.coords[0],
        delta_time,
        second_time,
    );
    (
        two_three.0 + one_one.0 - two_one.0 - one_three.0, // g_2
        two_three.1 + one_one.1 - two_one.1 - one_three.1, // g_1
        two_three.2 + one_one.2 - two_one.2 - one_three.2, // g_0
    )
}

/// calculate the sub cross product parameters (called f_{n, a, b} in the thesis).
fn surface_sub_cross_product_parameters(
    coords_a_first: &Vector3<f32>,
    coords_a_second: &Vector3<f32>,
    coords_b_first: &Vector3<f32>,
    coords_b_second: &Vector3<f32>,
    delta_time: f32,
    second_time: f32,
) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
    let a1b1 = coords_a_first.cross(coords_b_first);
    let a1b2 = coords_a_first.cross(coords_b_second);
    let a2b1 = coords_a_second.cross(coords_b_first);
    let a2b2 = coords_a_second.cross(coords_b_second);

    (
        a1b1 - a2b1 - a1b2 + a2b2, //f_{2, a, b}
        -delta_time * (-2f32 * a1b1 + a2b1 + a1b2)
            - 2f32 * second_time * (a1b1 - a2b1 - a1b2 + a2b2), // f_{1, a, b}
        delta_time.powi(2) * a1b1
            + second_time * delta_time * (-2f32 * a1b1 + a2b1 + a1b2)
            + second_time.powi(2) * (a1b1 - a2b1 - a1b2 + a2b2), // f_{0, a, b}
    )
}

/// Check whether the given surface intersects with the given ray.
/// This check is pretty trivial - first calculating an intersection
/// by determining a time such that the ray is hitting the plane the
/// triangle is in at that point, then checking whether that point is
/// inside the triangle itself using barycentric coordinates.
fn intersection_check_surface_coordinates(
    ray: &Ray,
    coords: &[Vector3<f32>; 3],
    time_entry: u32,
    time_exit: u32,
) -> Option<(u32, Vector3<f32>)> {
    let normal = (coords[1] - coords[0]).cross(&(coords[2] - coords[0]));
    let direction_dot_normal = ray.direction.into_inner().dot(&normal);
    if direction_dot_normal == 0f32 {
        return None;
    }
    let intersection_time =
        -(ray.origin - coords[0]).dot(&normal) / (ray.velocity * direction_dot_normal);
    if (intersection_time.trunc() as u32) < time_entry
        || intersection_time.ceil() as u32 > time_exit
    {
        return None;
    }

    let ray_coords = ray.coords_at_time(intersection_time);

    if maths::is_point_inside_triangle(&ray_coords, coords) {
        Some((intersection_time.round() as u32, ray_coords))
    } else {
        None
    }
}

/// Find the first intersection between the given ray and receiver.
/// The intersection logic for interpolated/keyframe receiver is defined in
/// `intersection_check_receiver_coordinates` and `intersection_check_receiver_keyframes`
/// respectively.
/// For interpolated receivers, only one check is required because they don't change. For keyframe
/// receivers, a check between every set of keyframes relevant to the entry/exit time is done.
pub fn intersect_ray_and_receiver(
    ray: &Ray,
    receiver: &Receiver,
    time_entry: u32,
    time_exit: u32,
) -> Option<(u32, Vector3<f32>)> {
    match receiver {
        Receiver::Interpolated(coords, radius, _time) => {
            intersection_check_receiver_coordinates(ray, coords, *radius, time_entry, time_exit)
        }
        Receiver::Keyframes(keyframes, radius) => {
            for pair in keyframes.windows(2) {
                if pair[1].time < time_entry {
                    continue;
                }
                if pair[0].time > time_exit {
                    return None;
                }
                if let Some((time, coords)) =
                    intersection_check_receiver_keyframes(ray, &pair[0], &pair[1], *radius)
                {
                    return Some((time, coords));
                }
            }
            // do final check after last keyframe
            let final_keyframe = &keyframes[keyframes.len() - 1];
            intersection_check_receiver_coordinates(
                ray,
                &final_keyframe.coords,
                *radius,
                final_keyframe.time,
                time_exit,
            )
        }
    }
}

/// Check for an intersection inbetween the two given keyframes.
/// This uses the logic explained in the "Intersection Checks" chapter of the thesis,
/// with its corresponding variable names.
fn intersection_check_receiver_keyframes(
    ray: &Ray,
    keyframe_first: &CoordinateKeyframe,
    keyframe_second: &CoordinateKeyframe,
    radius: f32,
) -> Option<(u32, Vector3<f32>)> {
    let (d2, d1, d0) = receiver_polynomial_parameters(ray, keyframe_first, keyframe_second, radius);
    let intersections = maths::solve_quadratic_equation(d2, d1, d0);
    let mut intersection: Option<f32> = None;
    for intersection_time in intersections {
        if (intersection_time.floor() as u32) < keyframe_first.time
            || intersection_time.ceil() as u32 > keyframe_second.time
        {
            continue;
        }
        if match intersection {
            Some(time) => time > intersection_time,
            None => true,
        } {
            intersection = Some(intersection_time);
        }
    }

    let Some(intersection_time) = intersection else {
        return None;
    };

    let ray_coords = ray.coords_at_time(intersection_time);

    Some((intersection_time.round() as u32, ray_coords))
}

/// Calculate the sphere intersection polynomial parameters (called d_0 through d_2 in the thesis).
fn receiver_polynomial_parameters(
    ray: &Ray,
    keyframe_first: &CoordinateKeyframe,
    keyframe_second: &CoordinateKeyframe,
    radius: f32,
) -> (f32, f32, f32) {
    let velocity = ray.velocity * ray.direction.into_inner();
    let delta_time = (keyframe_second.time - keyframe_first.time) as f32;
    let second_time = keyframe_second.time as f32;
    let diff_center = keyframe_first.coords - keyframe_second.coords;
    let ray_origin_to_center_first = ray.origin - keyframe_first.coords;
    (
        velocity.norm_squared() * delta_time.powi(2) + diff_center.norm_squared()
            - 2f32 * delta_time * velocity.dot(&diff_center), // d_2
        2f32 * (delta_time.powi(2) * ray_origin_to_center_first.dot(&velocity)
            - delta_time * ray_origin_to_center_first.dot(&diff_center)
            + second_time * delta_time * velocity.dot(&diff_center)
            - second_time * diff_center.norm_squared()), // d_1
        ray_origin_to_center_first.norm_squared() * delta_time.powi(2)
            + 2f32 * second_time * delta_time * ray_origin_to_center_first.dot(&diff_center)
            + second_time.powi(2) * diff_center.norm_squared()
            - radius.powi(2) * delta_time.powi(2), // d_0
    )
}

/// Check for an intersection between the receiver (as a static sphere) and
/// the ray.
fn intersection_check_receiver_coordinates(
    ray: &Ray,
    coords: &Vector3<f32>,
    radius: f32,
    time_entry: u32,
    time_exit: u32,
) -> Option<(u32, Vector3<f32>)> {
    let origin_to_coords = coords - ray.origin;
    let time_origin_to_angle = origin_to_coords.dot(&(ray.direction.into_inner() * ray.velocity));
    if time_origin_to_angle < 0f32 {
        return None;
    }
    let time_coords_to_angle = (origin_to_coords.dot(&origin_to_coords)
        - time_origin_to_angle.powi(2))
    .abs()
    .sqrt();
    if radius - time_coords_to_angle < -0.0001 {
        // rounding errors
        return None;
    }
    let time_angle_to_result = (radius.powi(2) - time_coords_to_angle.powi(2)).abs().sqrt();
    let intersection_time = time_origin_to_angle - time_angle_to_result;

    if (intersection_time.trunc() as u32) < time_entry
        || intersection_time.ceil() as u32 > time_exit
    {
        return None;
    }

    let ray_coords = ray.coords_at_time(intersection_time);

    Some((intersection_time.round() as u32, ray_coords))
}
