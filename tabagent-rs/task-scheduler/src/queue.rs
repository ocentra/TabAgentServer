//! Priority-based task queue.
//!
//! Tasks are organized by priority and executed based on current activity level.

use crate::tasks::Task;
use std::collections::VecDeque;

/// Priority level for tasks.
///
/// Higher priority tasks run first, and may run even during high activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// Critical tasks that must run immediately (e.g., indexing new message for instant recall).
    ///
    /// Runs even during HighActivity.
    Urgent = 3,
    
    /// Normal background tasks (e.g., generating embeddings).
    ///
    /// Runs during LowActivity and SleepMode.
    Normal = 2,
    
    /// Low priority tasks (e.g., entity extraction, linking).
    ///
    /// Runs during LowActivity and SleepMode, after Normal tasks.
    Low = 1,
    
    /// Batch tasks that can wait (e.g., summarization, associative linking).
    ///
    /// Runs during SleepMode only, when system has plenty of time.
    Batch = 0,
}

/// A priority-based task queue.
pub struct TaskQueue {
    urgent: VecDeque<Task>,
    normal: VecDeque<Task>,
    low: VecDeque<Task>,
    batch: VecDeque<Task>,
}

impl TaskQueue {
    /// Creates a new empty task queue.
    pub fn new() -> Self {
        Self {
            urgent: VecDeque::new(),
            normal: VecDeque::new(),
            low: VecDeque::new(),
            batch: VecDeque::new(),
        }
    }
    
    /// Adds a task to the appropriate priority queue.
    pub fn push(&mut self, task: Task) {
        match task.priority() {
            TaskPriority::Urgent => self.urgent.push_back(task),
            TaskPriority::Normal => self.normal.push_back(task),
            TaskPriority::Low => self.low.push_back(task),
            TaskPriority::Batch => self.batch.push_back(task),
        }
    }
    
    /// Pops the highest priority urgent task.
    ///
    /// Used during HighActivity when only critical tasks should run.
    pub fn pop_urgent(&mut self) -> Option<Task> {
        self.urgent.pop_front()
    }
    
    /// Pops the highest priority task from any queue.
    ///
    /// Priority order: Urgent → Normal → Low → Batch
    pub fn pop_any(&mut self) -> Option<Task> {
        self.urgent.pop_front()
            .or_else(|| self.normal.pop_front())
            .or_else(|| self.low.pop_front())
            .or_else(|| self.batch.pop_front())
    }
    
    /// Returns statistics about the queue.
    pub fn stats(&self) -> crate::QueueStats {
        crate::QueueStats {
            urgent_count: self.urgent.len(),
            normal_count: self.normal.len(),
            low_count: self.low.len(),
            batch_count: self.batch.len(),
        }
    }
    
    /// Returns the total number of pending tasks.
    pub fn len(&self) -> usize {
        self.urgent.len() + self.normal.len() + self.low.len() + self.batch.len()
    }
    
    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Clears all tasks from the queue.
    pub fn clear(&mut self) {
        self.urgent.clear();
        self.normal.clear();
        self.low.clear();
        self.batch.clear();
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::NodeId;
    
    fn create_test_task(priority: TaskPriority) -> Task {
        Task::GenerateEmbedding {
            node_id: NodeId::from("test"),
            text: "test".to_string(),
            priority,
        }
    }
    
    #[test]
    fn test_priority_ordering() {
        let mut queue = TaskQueue::new();
        
        // Add tasks in reverse priority order
        queue.push(create_test_task(TaskPriority::Batch));
        queue.push(create_test_task(TaskPriority::Normal));
        queue.push(create_test_task(TaskPriority::Urgent));
        queue.push(create_test_task(TaskPriority::Low));
        
        // Pop should return in priority order
        let task1 = queue.pop_any().unwrap();
        assert_eq!(task1.priority(), TaskPriority::Urgent);
        
        let task2 = queue.pop_any().unwrap();
        assert_eq!(task2.priority(), TaskPriority::Normal);
        
        let task3 = queue.pop_any().unwrap();
        assert_eq!(task3.priority(), TaskPriority::Low);
        
        let task4 = queue.pop_any().unwrap();
        assert_eq!(task4.priority(), TaskPriority::Batch);
    }
    
    #[test]
    fn test_urgent_only_during_high_activity() {
        let mut queue = TaskQueue::new();
        
        queue.push(create_test_task(TaskPriority::Normal));
        queue.push(create_test_task(TaskPriority::Urgent));
        
        // During high activity, only urgent pops
        let task = queue.pop_urgent().unwrap();
        assert_eq!(task.priority(), TaskPriority::Urgent);
        
        // Normal task is still queued
        assert!(queue.pop_urgent().is_none());
        assert_eq!(queue.stats().normal_count, 1);
    }
    
    #[test]
    fn test_queue_stats() {
        let mut queue = TaskQueue::new();
        
        queue.push(create_test_task(TaskPriority::Urgent));
        queue.push(create_test_task(TaskPriority::Urgent));
        queue.push(create_test_task(TaskPriority::Normal));
        queue.push(create_test_task(TaskPriority::Batch));
        
        let stats = queue.stats();
        assert_eq!(stats.urgent_count, 2);
        assert_eq!(stats.normal_count, 1);
        assert_eq!(stats.low_count, 0);
        assert_eq!(stats.batch_count, 1);
    }
}

