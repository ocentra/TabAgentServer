//! Background task scheduler with activity-aware execution.
//!
//! This module provides a task queue that intelligently schedules background work
//! based on user activity levels. Inspired by human brain behavior:
//! - During active use: Only critical tasks run
//! - During idle time: Normal background processing
//! - During sleep/inactivity: Aggressive batch processing
//!
//! # Architecture
//!
//! ```text
//! User Activity → Activity Detector → Task Scheduler → Task Queue
//!                                            ↓
//!                                    Executor (tokio tasks)
//! ```
//!
//! # Example
//!
//! ```no_run
//! use task_scheduler::{TaskScheduler, Task, TaskPriority, ActivityLevel};
//! use common::NodeId;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let scheduler = TaskScheduler::new();
//! 
//! // User starts chatting - switch to high activity mode
//! scheduler.set_activity(ActivityLevel::HighActivity).await;
//!
//! // Queue a task (will wait if high activity and low priority)
//! scheduler.submit(Task::GenerateEmbedding {
//!     node_id: NodeId::from("msg_123"),
//!     text: "Hello world".to_string(),
//!     priority: TaskPriority::Normal,
//! }).await?;
//!
//! // User goes idle - tasks start processing
//! scheduler.set_activity(ActivityLevel::LowActivity).await;
//! # Ok(())
//! # }
//! ```

pub mod activity;
pub mod queue;
pub mod tasks;

pub use activity::{ActivityDetector, ActivityLevel};
pub use queue::{TaskQueue, TaskPriority};
pub use tasks::{Task, TaskResult, TaskError};

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::time;

/// The main task scheduler that coordinates background work based on activity levels.
pub struct TaskScheduler {
    /// Current activity level
    activity: Arc<RwLock<ActivityLevel>>,
    
    /// Task queue
    queue: Arc<Mutex<TaskQueue>>,
    
    /// Channel for submitting tasks
    task_tx: mpsc::UnboundedSender<Task>,
    
    /// Channel for activity changes
    activity_tx: mpsc::UnboundedSender<ActivityLevel>,
    
    /// Running state
    running: Arc<RwLock<bool>>,
}

impl TaskScheduler {
    /// Creates a new task scheduler and starts the background executor.
    pub fn new() -> Self {
        let (task_tx, task_rx) = mpsc::unbounded_channel();
        let (activity_tx, activity_rx) = mpsc::unbounded_channel();
        
        let activity = Arc::new(RwLock::new(ActivityLevel::LowActivity));
        let queue = Arc::new(Mutex::new(TaskQueue::new()));
        let running = Arc::new(RwLock::new(true));
        
        // Spawn background executor
        let executor = TaskExecutor {
            task_rx,
            activity_rx,
            activity: activity.clone(),
            queue: queue.clone(),
            running: running.clone(),
        };
        
        tokio::spawn(async move {
            executor.run().await;
        });
        
        Self {
            activity,
            queue,
            task_tx,
            activity_tx,
            running,
        }
    }
    
    /// Submits a task for background processing.
    pub async fn submit(&self, task: Task) -> Result<(), TaskError> {
        self.task_tx.send(task)
            .map_err(|_| TaskError::SchedulerShutdown)?;
        Ok(())
    }
    
    /// Updates the current activity level.
    ///
    /// - `HighActivity`: User is actively chatting - only urgent tasks run
    /// - `LowActivity`: User is idle - normal processing
    /// - `SleepMode`: System inactive - aggressive batch processing
    pub async fn set_activity(&self, level: ActivityLevel) {
        let _ = self.activity_tx.send(level);
    }
    
    /// Gets the current activity level.
    pub async fn get_activity(&self) -> ActivityLevel {
        *self.activity.read().await
    }
    
    /// Returns the number of pending tasks in each priority level.
    pub async fn queue_stats(&self) -> QueueStats {
        self.queue.lock().await.stats()
    }
    
    /// Gracefully shuts down the scheduler.
    pub async fn shutdown(&self) {
        *self.running.write().await = false;
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the task queue.
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub urgent_count: usize,
    pub normal_count: usize,
    pub low_count: usize,
    pub batch_count: usize,
}

/// Internal executor that processes tasks from the queue.
struct TaskExecutor {
    task_rx: mpsc::UnboundedReceiver<Task>,
    activity_rx: mpsc::UnboundedReceiver<ActivityLevel>,
    activity: Arc<RwLock<ActivityLevel>>,
    queue: Arc<Mutex<TaskQueue>>,
    running: Arc<RwLock<bool>>,
}

impl TaskExecutor {
    async fn run(mut self) {
        let mut tick_interval = time::interval(Duration::from_millis(100));
        
        loop {
            tokio::select! {
                // Receive new tasks
                Some(task) = self.task_rx.recv() => {
                    self.queue.lock().await.push(task);
                }
                
                // Receive activity changes
                Some(level) = self.activity_rx.recv() => {
                    *self.activity.write().await = level;
                    println!("[TaskScheduler] Activity changed to {:?}", level);
                }
                
                // Process tasks periodically
                _ = tick_interval.tick() => {
                    if !*self.running.read().await {
                        break;
                    }
                    
                    self.process_tasks().await;
                }
            }
        }
        
        println!("[TaskScheduler] Executor shutting down");
    }
    
    async fn process_tasks(&self) {
        let activity = *self.activity.read().await;
        let mut queue = self.queue.lock().await;
        
        // Determine how many tasks to process based on activity
        let max_tasks = match activity {
            ActivityLevel::HighActivity => 1,      // Only 1 urgent task at a time
            ActivityLevel::LowActivity => 5,       // Normal processing
            ActivityLevel::SleepMode => 100,       // Aggressive batch processing
        };
        
        let mut processed = 0;
        
        while processed < max_tasks {
            // Pop next task based on activity level
            let task = match activity {
                ActivityLevel::HighActivity => queue.pop_urgent(),
                ActivityLevel::LowActivity => queue.pop_any(),
                ActivityLevel::SleepMode => queue.pop_any(),
            };
            
            if let Some(task) = task {
                // Spawn task in background (don't block executor)
                tokio::spawn(async move {
                    match task.execute().await {
                        Ok(result) => {
                            println!("[Task] Completed: {}", task.name());
                            if let Some(res) = result {
                                println!("  Result: {:?}", res);
                            }
                        }
                        Err(e) => {
                            eprintln!("[Task] Failed: {} - {:?}", task.name(), e);
                        }
                    }
                });
                
                processed += 1;
            } else {
                // No more tasks to process
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::NodeId;
    
    #[tokio::test]
    async fn test_scheduler_activity_levels() {
        let scheduler = TaskScheduler::new();
        
        // Start with low activity
        assert!(matches!(scheduler.get_activity().await, ActivityLevel::LowActivity));
        
        // Switch to high activity
        scheduler.set_activity(ActivityLevel::HighActivity).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(matches!(scheduler.get_activity().await, ActivityLevel::HighActivity));
        
        // Switch to sleep mode
        scheduler.set_activity(ActivityLevel::SleepMode).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(matches!(scheduler.get_activity().await, ActivityLevel::SleepMode));
    }
    
    #[tokio::test]
    async fn test_task_submission() {
        let scheduler = TaskScheduler::new();
        
        // Switch to high activity so task queues instead of executing
        scheduler.set_activity(ActivityLevel::HighActivity).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Submit a normal priority task (won't execute during high activity)
        let task = Task::GenerateEmbedding {
            node_id: NodeId::from("test_123"),
            text: "Test message".to_string(),
            priority: TaskPriority::Normal,
        };
        
        scheduler.submit(task).await.expect("Failed to submit task");
        
        // Give it time to queue
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Task should still be queued (not executed during high activity)
        let stats = scheduler.queue_stats().await;
        assert_eq!(stats.normal_count, 1);
    }
}
