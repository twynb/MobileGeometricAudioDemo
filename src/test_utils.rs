use std::collections::HashSet;
use std::hash::Hash;
use nalgebra::Vector3;
use approx::abs_diff_eq;

/// Check that two unordered collections of items are equal.
/// This ignores duplicates within the collections!
/// by [StackOverflow user Shepmaster](https://stackoverflow.com/a/42748484/16293155)
pub fn unordered_eq_without_duplicates<T: Eq + Hash>(a: &[T], b: &[T]) -> bool {
    let a: HashSet<_> = a.iter().collect();
    let b: HashSet<_> = b.iter().collect();
    a == b
}

/// Assert that the given two unordered collections of items are equal.
/// This ignores duplicates within the collections!
pub fn assert_unordered_eq_ignoring_duplicates<T: Eq + Hash>(a: &[T], b: &[T]) {
    assert!(unordered_eq_without_duplicates(a, b));
}

pub fn vector_abs_diff_eq(a: Vector3<f64>, b: Vector3<f64>) -> bool {
    for i in 0..3 {
        if !(abs_diff_eq!(a[i], b[i], epsilon=0.000001)) {
            return false;
        }
    }
    true
}

pub fn assert_vector_abs_diff_eq(a: Vector3<f64>, b: Vector3<f64>) {
    assert!(vector_abs_diff_eq(a, b), "assertion `left == right` failed. left: {a:?}, right: {b:?}");
}