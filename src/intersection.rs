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
#[allow(clippy::option_if_let_else)]
pub fn intersect_ray_and_surface(
    ray: &Ray,
    surface: &Surface<3>,
    time_entry: u32,
    time_exit: u32,
    scene_looping_duration: Option<u32>,
) -> Option<(f64, Vector3<f64>)> {
    match surface {
        Surface::Interpolated(coords, _time, _material) => {
            intersection_check_surface_coordinates(ray, coords, time_entry, time_exit)
        }
        Surface::Keyframes(keyframes, _material) => match scene_looping_duration {
            Some(loop_duration) => intersection_check_surface_looping(
                ray,
                keyframes,
                time_entry,
                time_exit,
                loop_duration,
            ),
            None => intersection_check_surface_non_looping(ray, keyframes, time_entry, time_exit),
        },
    }
}

fn intersection_check_surface_non_looping(
    ray: &Ray,
    keyframes: &[SurfaceKeyframe<3>],
    time_entry: u32,
    time_exit: u32,
) -> Option<(f64, Vector3<f64>)> {
    for pair in keyframes.windows(2) {
        if pair[1].time < time_entry {
            continue;
        }
        if pair[0].time > time_exit {
            return None;
        }
        if let Some((time, coords)) = intersection_check_surface_keyframes(
            ray,
            &pair[0],
            &pair[1],
            std::cmp::max(time_entry, pair[0].time),
            std::cmp::min(time_exit, pair[1].time),
            0,
        ) {
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

fn intersection_check_surface_looping(
    ray: &Ray,
    keyframes: &[SurfaceKeyframe<3>],
    time_entry: u32,
    time_exit: u32,
    loop_duration: u32,
) -> Option<(f64, Vector3<f64>)> {
    // round start time to last looping time
    let mut current_time = time_entry - (time_entry % loop_duration);
    while current_time <= time_exit {
        for pair in keyframes.windows(2) {
            if current_time + pair[1].time < time_entry {
                continue;
            }
            if current_time + pair[0].time > time_exit {
                return None;
            }
            if let Some((time, coords)) = intersection_check_surface_keyframes(
                ray,
                &pair[0],
                &pair[1],
                std::cmp::max(time_entry, current_time + pair[0].time),
                std::cmp::min(time_exit, current_time + pair[1].time),
                current_time,
            ) {
                return Some((time, coords));
            }
        }
        // do final check for loop after last keyframe
        let final_keyframe = &keyframes[keyframes.len() - 1];
        if final_keyframe.time < loop_duration {
            if let Some((time, coords)) = intersection_check_surface_coordinates(
                ray,
                &final_keyframe.coords,
                current_time + final_keyframe.time,
                current_time + loop_duration,
            ) {
                return Some((time, coords));
            }
        }
        current_time += loop_duration;
    }
    None
}

/// Check for an intersection inbetween the two given keyframes.
/// This uses the logic explained in the "Intersection Checks" chapter of the thesis,
/// with its corresponding variable names.
fn intersection_check_surface_keyframes(
    ray: &Ray,
    keyframe_first: &SurfaceKeyframe<3>,
    keyframe_second: &SurfaceKeyframe<3>,
    time_entry: u32,
    time_exit: u32,
    loop_offset: u32,
) -> Option<(f64, Vector3<f64>)> {
    let (d3, d2, d1, d0) =
        surface_polynomial_parameters(ray, keyframe_first, keyframe_second, loop_offset);

    // roots::find_roots_cubic() gets *badly* inaccurate if d3 is a lot smaller than d0..2
    // so if that's the case, we'll rather just do quadratic - the difference d3 makes is ignorable anyway
    let intersections = if d3 == 0f64 || d2.abs().log10() - d3.abs().log10() > 7f64 {
        roots::find_roots_quadratic(d2, d1, d0)
    } else {
        roots::find_roots_cubic(d3, d2, d1, d0)
    };
    let mut intersection: Option<(f64, Vector3<f64>)> = None;
    for intersection_time in intersections.as_ref() {
        if *intersection_time < 0f64
            || (intersection_time.floor() as u32) < time_entry
            || (intersection_time.ceil() as u32) > time_exit
        {
            continue;
        }

        if match intersection {
            Some((time, _coords)) => time > *intersection_time,
            None => true,
        } {
            let Some(surface_coords) = interpolate_two_surface_keyframes(
                keyframe_first,
                keyframe_second,
                *intersection_time - f64::from(loop_offset),
            ) else {
                continue;
            };

            let normal = (surface_coords[2] - surface_coords[0])
                .cross(&(surface_coords[1] - surface_coords[0]));

            if normal.dot(&ray.direction) > 0f64 {
                // we are behind the surface, so just skip
                continue;
            }

            let ray_coords = ray.coords_at_time(*intersection_time);

            if maths::is_point_inside_triangle(&ray_coords, &surface_coords) {
                intersection = Some((*intersection_time, ray_coords));
            }
        }
    }

    intersection
}

/// Calculate the surface intersection polynomial parameters (called `d_0` through `d_3` in the thesis).
fn surface_polynomial_parameters(
    ray: &Ray,
    keyframe_first: &SurfaceKeyframe<3>,
    keyframe_second: &SurfaceKeyframe<3>,
    loop_offset: u32,
) -> (f64, f64, f64, f64) {
    let second_time = f64::from(keyframe_second.time + loop_offset); // t_k_2
    let (g2, g1, g0) =
        surface_cross_product_parameters(keyframe_first, keyframe_second, second_time);
    let ray_time = ray.time; // t_0
    let velocity = ray.velocity * ray.direction.into_inner();
    let delta_time = f64::from(keyframe_second.time - keyframe_first.time);
    let delta_point_1 = keyframe_second.coords[0] - keyframe_first.coords[0];
    let g2_dot_delta_p1_div_delta_time = g2.dot(&delta_point_1) / delta_time;
    let g1_dot_delta_p1_div_delta_time = g1.dot(&delta_point_1) / delta_time;
    let g0_dot_delta_p1_div_delta_time = g0.dot(&delta_point_1) / delta_time;
    let g2_dot_velocity = g2.dot(&velocity);
    let g1_dot_velocity = g1.dot(&velocity);
    let g0_dot_velocity = g0.dot(&velocity);
    /*
       // Non-optimised (readable) version
       (
           g2_dot_velocity - g2_dot_delta_p1_div_delta_time, // d_3
           g2.dot(&ray.origin) - ray_time * g2_dot_velocity - g2.dot(&keyframe_second.coords[0])
               + g2_dot_delta_p1_div_delta_time * f64::from(keyframe_second.time)
               + g1_dot_velocity
               - g1_dot_delta_p1_div_delta_time, // d_2
           g1.dot(&ray.origin) - ray_time * g1_dot_velocity - g1.dot(&keyframe_second.coords[0])
               + g1_dot_delta_p1_div_delta_time * f64::from(keyframe_second.time)
               + g0_dot_velocity
               - g0_dot_delta_p1_div_delta_time, // d_1
           g0.dot(&ray.origin) - ray_time * g0_dot_velocity - g0.dot(&keyframe_second.coords[0])
               + g0_dot_delta_p1_div_delta_time * f64::from(keyframe_second.time), // d_0
       )
    */
    (
        g2_dot_velocity - g2_dot_delta_p1_div_delta_time, // d_3
        ray_time.mul_add(
            -g2_dot_velocity,
            g2.dot(&ray.origin) - g2.dot(&keyframe_second.coords[0])
                + g2_dot_delta_p1_div_delta_time.mul_add(
                    second_time,
                    g1_dot_velocity - g1_dot_delta_p1_div_delta_time,
                ),
        ), // d_2
        ray_time.mul_add(
            -g1_dot_velocity,
            g1.dot(&ray.origin) - g1.dot(&keyframe_second.coords[0])
                + g1_dot_delta_p1_div_delta_time.mul_add(
                    second_time,
                    g0_dot_velocity - g0_dot_delta_p1_div_delta_time,
                ),
        ), // d_1
        ray_time.mul_add(
            -g0_dot_velocity,
            g0_dot_delta_p1_div_delta_time.mul_add(
                second_time,
                g0.dot(&ray.origin) - g0.dot(&keyframe_second.coords[0]),
            ),
        ), // d_0
    )
}

/// Calculate the cross product parameters (called `g_0` through `g_2` in the thesis).
fn surface_cross_product_parameters(
    keyframe_first: &SurfaceKeyframe<3>,
    keyframe_second: &SurfaceKeyframe<3>,
    second_time: f64,
) -> (Vector3<f64>, Vector3<f64>, Vector3<f64>) {
    let delta_time = f64::from(keyframe_second.time - keyframe_first.time);
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
    (
        two_three.0 - two_one.0 - one_three.0, // g_2
        two_three.1 - two_one.1 - one_three.1, // g_1
        two_three.2 - two_one.2 - one_three.2, // g_0
    )
}

/// calculate the sub cross product parameters (called f_{n, a, b} in the thesis).
#[allow(clippy::similar_names)]
fn surface_sub_cross_product_parameters(
    coords_a_first: &Vector3<f64>,
    coords_a_second: &Vector3<f64>,
    coords_b_first: &Vector3<f64>,
    coords_b_second: &Vector3<f64>,
    delta_time: f64,
    second_time: f64,
) -> (Vector3<f64>, Vector3<f64>, Vector3<f64>) {
    let a1b1 = coords_a_first.cross(coords_b_first);
    let a1b2 = coords_a_first.cross(coords_b_second);
    let a2b1 = coords_a_second.cross(coords_b_first);
    let a2b2 = coords_a_second.cross(coords_b_second);

    (
        a2b2 - a1b2 - a2b1 + a1b1, //f_{2, a, b}
        -delta_time * (-2f64 * a2b2 + a1b2 + a2b1)
            - 2f64 * second_time * (a2b2 - a1b2 - a2b1 + a1b1), // f_{1, a, b}
        delta_time.powi(2) * a2b2
            + second_time * delta_time * (-2f64 * a2b2 + a1b2 + a2b1)
            + second_time.powi(2) * (a2b2 - a1b2 - a2b1 + a1b1), // f_{0, a, b}
    )
}

/// Check whether the given surface intersects with the given ray.
/// This check is pretty trivial - first calculating an intersection
/// by determining a time such that the ray is hitting the plane the
/// triangle is in at that point, then checking whether that point is
/// inside the triangle itself using barycentric coordinates.
fn intersection_check_surface_coordinates(
    ray: &Ray,
    coords: &[Vector3<f64>; 3],
    time_entry: u32,
    time_exit: u32,
) -> Option<(f64, Vector3<f64>)> {
    let normal = (coords[1] - coords[0]).cross(&(coords[2] - coords[0]));
    let direction_dot_normal = ray.direction.into_inner().dot(&normal);
    if direction_dot_normal == 0f64 {
        return None;
    }
    let intersection_time = -(ray.origin - coords[0]).dot(&normal)
        / (ray.velocity * direction_dot_normal)
        + ray.time;
    if (intersection_time.trunc() as u32) < time_entry
        || intersection_time.ceil() as u32 > time_exit
    {
        return None;
    }

    let normal = (coords[2] - coords[0]).cross(&(coords[1] - coords[0]));
    if normal.dot(&ray.direction) > 0f64 {
        return None;
    }

    let ray_coords = ray.coords_at_time(intersection_time);

    if maths::is_point_inside_triangle(&ray_coords, coords) {
        Some((intersection_time, ray_coords))
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
#[allow(clippy::option_if_let_else)]
pub fn intersect_ray_and_receiver(
    ray: &Ray,
    receiver: &Receiver,
    time_entry: u32,
    time_exit: u32,
    loop_duration: Option<u32>,
) -> Option<(f64, Vector3<f64>)> {
    match receiver {
        Receiver::Interpolated(coords, radius, _time) => {
            intersection_check_receiver_coordinates(ray, coords, *radius, time_entry, time_exit)
        }
        Receiver::Keyframes(keyframes, radius) => match loop_duration {
            Some(loop_time) => intersection_check_receiver_looping(
                ray, keyframes, time_entry, time_exit, *radius, loop_time,
            ),
            None => intersection_check_receiver_non_looping(
                ray, keyframes, time_entry, time_exit, *radius,
            ),
        },
    }
}

fn intersection_check_receiver_non_looping(
    ray: &Ray,
    keyframes: &[CoordinateKeyframe],
    time_entry: u32,
    time_exit: u32,
    radius: f64,
) -> Option<(f64, Vector3<f64>)> {
    for pair in keyframes.windows(2) {
        if pair[1].time < time_entry {
            continue;
        }
        if pair[0].time > time_exit {
            return None;
        }
        if let Some((time, coords)) = intersection_check_receiver_keyframes(
            ray,
            &pair[0],
            &pair[1],
            radius,
            std::cmp::max(time_entry, pair[0].time),
            std::cmp::min(time_exit, pair[1].time),
            0,
        ) {
            return Some((time, coords));
        }
    }
    // do final check after last keyframe
    let final_keyframe = &keyframes[keyframes.len() - 1];
    intersection_check_receiver_coordinates(
        ray,
        &final_keyframe.coords,
        radius,
        final_keyframe.time,
        time_exit,
    )
}

fn intersection_check_receiver_looping(
    ray: &Ray,
    keyframes: &[CoordinateKeyframe],
    time_entry: u32,
    time_exit: u32,
    radius: f64,
    loop_duration: u32,
) -> Option<(f64, Vector3<f64>)> {
    let mut current_time = time_entry - (time_entry % loop_duration);

    while current_time <= time_exit {
        for pair in keyframes.windows(2) {
            if current_time + pair[1].time < time_entry {
                continue;
            }
            if current_time + pair[0].time > time_exit {
                return None;
            }
            if let Some((time, coords)) = intersection_check_receiver_keyframes(
                ray,
                &pair[0],
                &pair[1],
                radius,
                std::cmp::max(time_entry, current_time + pair[0].time),
                std::cmp::min(time_exit, current_time + pair[1].time),
                current_time,
            ) {
                return Some((time, coords));
            }
        }
        // do final check after last keyframe
        let final_keyframe = &keyframes[keyframes.len() - 1];
        if final_keyframe.time < loop_duration {
            if let Some((time, coords)) = intersection_check_receiver_coordinates(
                ray,
                &final_keyframe.coords,
                radius,
                current_time + final_keyframe.time,
                current_time + loop_duration,
            ) {
                return Some((time, coords));
            }
        }
        current_time += loop_duration;
    }
    None
}

/// Check for an intersection inbetween the two given keyframes.
/// This uses the logic explained in the "Intersection Checks" chapter of the thesis,
/// with its corresponding variable names.
fn intersection_check_receiver_keyframes(
    ray: &Ray,
    keyframe_first: &CoordinateKeyframe,
    keyframe_second: &CoordinateKeyframe,
    radius: f64,
    time_entry: u32,
    time_exit: u32,
    loop_offset: u32,
) -> Option<(f64, Vector3<f64>)> {
    let (d2, d1, d0) =
        receiver_polynomial_parameters(ray, keyframe_first, keyframe_second, radius, loop_offset);
    let intersections = roots::find_roots_quadratic(d2, d1, d0);
    let mut intersection: Option<f64> = None;
    for intersection_time in intersections.as_ref() {
        if (intersection_time.floor() as u32) < time_entry
            || intersection_time.ceil() as u32 > time_exit
        {
            continue;
        }
        if intersection.map_or(true, |time| time > *intersection_time) {
            intersection = Some(*intersection_time);
        }
    }

    let Some(intersection_time) = intersection else {
        return None;
    };

    let ray_coords = ray.coords_at_time(intersection_time);

    Some((intersection_time, ray_coords))
}

/// Calculate the sphere intersection polynomial parameters (called `d_0` through `d_2` in the thesis).
fn receiver_polynomial_parameters(
    ray: &Ray,
    keyframe_first: &CoordinateKeyframe,
    keyframe_second: &CoordinateKeyframe,
    radius: f64,
    loop_offset: u32,
) -> (f64, f64, f64) {
    let p_minus_ck2 = ray.origin - keyframe_second.coords;
    let ray_time = ray.time;
    let velocity = ray.velocity * ray.direction.into_inner();
    let delta_time = f64::from(keyframe_second.time - keyframe_first.time);
    let delta_time_squared = delta_time.powi(2);
    let second_time = f64::from(keyframe_second.time + loop_offset);
    let delta_center = keyframe_second.coords - keyframe_first.coords;
    let p_minus_ck2_minus_t0_v = p_minus_ck2 - ray_time * velocity;
    let velocity_norm = velocity.norm_squared();
    let delta_center_norm = delta_center.norm_squared();
    let velocity_dot_delta_center = velocity.dot(&delta_center);
    let p_minus_ck2_minus_t0_v_dot_delta_center = p_minus_ck2_minus_t0_v.dot(&delta_center);
    /*
    // Unoptimized (readable) version:
    (
        delta_time_squared * velocity_norm + delta_center_norm
            - 2f64 * delta_time * velocity_dot_delta_center, // d_2
        2f64 * (delta_time_squared * p_minus_ck2_minus_t0_v.dot(&velocity)
            - delta_time * p_minus_ck2_minus_t0_v_dot_delta_center
            + second_time * delta_time * velocity_dot_delta_center
            - second_time * delta_center_norm), // d_1
        delta_time_squared
            * (p_minus_ck2.norm_squared()
                + 2f64 * ray_time * (-1f64 * p_minus_ck2).dot(&velocity)
                + ray_time.powi(2) * velocity_norm
                - radius.powi(2))
            + 2f64 * second_time * delta_time * p_minus_ck2_minus_t0_v_dot_delta_center
            + second_time.powi(2) * delta_center_norm, // d_0
    )
     */
    (
        delta_time_squared.mul_add(
            velocity_norm,
            delta_time.mul_add(-2f64 * velocity_dot_delta_center, delta_center_norm),
        ), // d_2
        2f64 * (delta_time_squared.mul_add(
            p_minus_ck2_minus_t0_v.dot(&velocity),
            p_minus_ck2_minus_t0_v_dot_delta_center.mul_add(
                -delta_time,
                second_time * delta_time.mul_add(velocity_dot_delta_center, -delta_center_norm),
            ),
        )), // d_1
        delta_time_squared.mul_add(
            2f64.mul_add(
                ray_time * (-1f64 * p_minus_ck2).dot(&velocity),
                velocity_norm.mul_add(
                    ray_time.powi(2),
                    radius.mul_add(-radius, p_minus_ck2.norm_squared()),
                ),
            ),
            delta_center_norm.mul_add(
                second_time.powi(2),
                2f64 * second_time * delta_time * p_minus_ck2_minus_t0_v_dot_delta_center,
            ),
        ), // d_0
    )
}

/// Check for an intersection between the receiver (as a static sphere) and
/// the ray.
fn intersection_check_receiver_coordinates(
    ray: &Ray,
    coords: &Vector3<f64>,
    radius: f64,
    time_entry: u32,
    time_exit: u32,
) -> Option<(f64, Vector3<f64>)> {
    let origin_to_coords = coords - ray.origin;
    let time_origin_to_angle = origin_to_coords.dot(&(ray.direction.into_inner()));
    if time_origin_to_angle < 0f64 {
        return None;
    }
    // non-optimised/readable version
    // let time_coords_to_angle = (origin_to_coords.norm_squared() - time_origin_to_angle.powi(2))
    let time_coords_to_angle = time_origin_to_angle
        .mul_add(-time_origin_to_angle, origin_to_coords.norm_squared())
        .abs()
        .sqrt();
    if radius - time_coords_to_angle < -0.0001 {
        // rounding errors
        return None;
    }
    // non-optimised/readable version
    // let time_angle_to_result = (radius.powi(2) - time_coords_to_angle.powi(2)).abs().sqrt();
    let time_angle_to_result = radius
        .mul_add(radius, -time_coords_to_angle.powi(2))
        .abs()
        .sqrt();
    let intersection_time =
        (time_origin_to_angle - time_angle_to_result) / ray.velocity + ray.time;

    if (intersection_time.trunc() as u32) < time_entry
        || intersection_time.ceil() as u32 > time_exit
    {
        return None;
    }

    let ray_coords = ray.coords_at_time(intersection_time);

    Some((intersection_time, ray_coords))
}
