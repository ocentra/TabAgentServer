//! Activity detection and management.
//!
//! This module defines activity levels and provides utilities for detecting
//! user activity patterns.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Represents the current level of user activity.
///
/// This affects how aggressively the task scheduler processes background tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityLevel {
    /// User is actively interacting (e.g., chatting, typing).
    ///
    /// **Behavior**: Only urgent tasks (critical indexes) run.
    /// Background tasks like embedding generation are paused.
    HighActivity,
    
    /// User is idle but TabAgent is in foreground.
    ///
    /// **Behavior**: Normal background processing. Embeddings, entity extraction,
    /// and other tasks run at a moderate pace.
    LowActivity,
    
    /// System is inactive (e.g., user switched tabs, minimized, or system sleep).
    ///
    /// **Behavior**: Aggressive batch processing. All pending tasks run as fast
    /// as possible to catch up while user isn't watching.
    SleepMode,
}

/// Detects user activity and automatically adjusts activity levels.
///
/// This is typically driven by UI events:
/// - Keypress, mouse click → HighActivity
/// - No input for 5 minutes → LowActivity  
/// - Tab/window inactive → SleepMode
pub struct ActivityDetector {
    last_activity: Instant,
    current_level: ActivityLevel,
    
    // Thresholds
    idle_threshold: Duration,
    sleep_threshold: Duration,
}

impl ActivityDetector {
    /// Creates a new activity detector with default thresholds.
    ///
    /// Default thresholds:
    /// - Idle after 5 minutes of no activity
    /// - Sleep mode after 30 minutes of no activity
    pub fn new() -> Self {
        Self {
            last_activity: Instant::now(),
            current_level: ActivityLevel::LowActivity,
            idle_threshold: Duration::from_secs(5 * 60),      // 5 minutes
            sleep_threshold: Duration::from_secs(30 * 60),    // 30 minutes
        }
    }
    
    /// Creates a detector with custom thresholds (useful for testing).
    pub fn with_thresholds(idle: Duration, sleep: Duration) -> Self {
        Self {
            last_activity: Instant::now(),
            current_level: ActivityLevel::LowActivity,
            idle_threshold: idle,
            sleep_threshold: sleep,
        }
    }
    
    /// Records user activity (called on UI events like keypress, mouse click).
    ///
    /// Returns the new activity level if it changed.
    pub fn record_activity(&mut self) -> Option<ActivityLevel> {
        self.last_activity = Instant::now();
        
        if self.current_level != ActivityLevel::HighActivity {
            self.current_level = ActivityLevel::HighActivity;
            Some(ActivityLevel::HighActivity)
        } else {
            None
        }
    }
    
    /// Manually sets the activity level (e.g., when tab becomes inactive).
    pub fn set_level(&mut self, level: ActivityLevel) -> Option<ActivityLevel> {
        if self.current_level != level {
            self.current_level = level;
            Some(level)
        } else {
            None
        }
    }
    
    /// Updates the activity level based on elapsed time since last activity.
    ///
    /// Call this periodically (e.g., every second) to auto-detect transitions
    /// from active → idle → sleep.
    ///
    /// Returns the new level if it changed.
    pub fn update(&mut self) -> Option<ActivityLevel> {
        let elapsed = self.last_activity.elapsed();
        
        let new_level = if elapsed >= self.sleep_threshold {
            ActivityLevel::SleepMode
        } else if elapsed >= self.idle_threshold {
            ActivityLevel::LowActivity
        } else {
            // Still within active window, don't change to HighActivity
            // (that only happens on actual user input)
            return None;
        };
        
        if new_level != self.current_level {
            self.current_level = new_level;
            Some(new_level)
        } else {
            None
        }
    }
    
    /// Returns the current activity level.
    pub fn current_level(&self) -> ActivityLevel {
        self.current_level
    }
    
    /// Returns the time since last recorded activity.
    pub fn time_since_activity(&self) -> Duration {
        self.last_activity.elapsed()
    }
}

impl Default for ActivityDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_activity_detection() {
        let mut detector = ActivityDetector::with_thresholds(
            Duration::from_millis(100),  // Idle after 100ms
            Duration::from_millis(200),  // Sleep after 200ms
        );
        
        // Initial state
        assert_eq!(detector.current_level(), ActivityLevel::LowActivity);
        
        // Record activity
        let changed = detector.record_activity();
        assert_eq!(changed, Some(ActivityLevel::HighActivity));
        assert_eq!(detector.current_level(), ActivityLevel::HighActivity);
        
        // Wait for idle threshold
        thread::sleep(Duration::from_millis(150));
        let changed = detector.update();
        assert_eq!(changed, Some(ActivityLevel::LowActivity));
        
        // Wait for sleep threshold
        thread::sleep(Duration::from_millis(100));
        let changed = detector.update();
        assert_eq!(changed, Some(ActivityLevel::SleepMode));
    }
    
    #[test]
    fn test_manual_level_setting() {
        let mut detector = ActivityDetector::new();
        
        // Manually set to sleep mode (e.g., window minimized)
        let changed = detector.set_level(ActivityLevel::SleepMode);
        assert_eq!(changed, Some(ActivityLevel::SleepMode));
        assert_eq!(detector.current_level(), ActivityLevel::SleepMode);
        
        // Setting same level returns None
        let changed = detector.set_level(ActivityLevel::SleepMode);
        assert_eq!(changed, None);
    }
}

