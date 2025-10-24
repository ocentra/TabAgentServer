//! Tests for DatabaseType and TemperatureTier

use storage::{DatabaseType, TemperatureTier};

#[test]
fn test_database_type_paths() {
    let path = DatabaseType::Conversations.get_path(Some(TemperatureTier::Active));
    assert!(path.to_string_lossy().contains("conversations"));
    assert!(path.to_string_lossy().contains("active"));
}

#[test]
fn test_source_databases() {
    assert!(DatabaseType::Conversations.is_source());
    assert!(DatabaseType::Experience.is_source());
    assert!(!DatabaseType::Knowledge.is_source());
}

#[test]
fn test_default_tiers() {
    let conv_tiers = DatabaseType::Conversations.default_tiers();
    assert_eq!(conv_tiers.len(), 3);
    assert!(conv_tiers.contains(&TemperatureTier::Active));
}
