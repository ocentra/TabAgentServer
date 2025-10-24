//! Tests for the TimeScope enum and its implementations

use storage::time_scope::TimeScope;

#[test]
fn test_time_scope_contains() {
    let current_time = 1700000000000i64; // Some timestamp

    // Test Today scope
    let today = TimeScope::Today;
    assert!(today.contains(current_time, current_time));
    assert!(!today.contains(current_time - 25 * 60 * 60 * 1000, current_time)); // 25 hours ago

    // Test ThisWeek scope
    let this_week = TimeScope::ThisWeek;
    assert!(this_week.contains(current_time, current_time));
    assert!(!this_week.contains(current_time - 8 * 24 * 60 * 60 * 1000, current_time));
    // 8 days ago
}

#[test]
fn test_time_scope_get_time_range() {
    let current_time = 1700000000000i64;

    // Test Today scope range
    let today = TimeScope::Today;
    let (start, end) = today.get_time_range(current_time);
    assert_eq!(end, current_time);
    assert_eq!(start, current_time - 24 * 60 * 60 * 1000);

    // Test Custom scope
    let custom = TimeScope::Custom(1000, 2000);
    let (start, end) = custom.get_time_range(current_time);
    assert_eq!(start, 1000);
    assert_eq!(end, 2000);
}
