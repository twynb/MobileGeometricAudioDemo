use std::collections::HashSet;
use std::hash::Hash;

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
