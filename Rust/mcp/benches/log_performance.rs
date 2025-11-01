//! Benchmarks for MCP log operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tempfile::TempDir;
use std::sync::Arc;

use tabagent_mcp::McpManager;
use common::logging::{LogEntry, LogLevel, LogQuery, LogSource};
use storage::DatabaseCoordinator;
use appstate::{AppState, AppStateConfig};

async fn setup_bench_manager() -> (McpManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    let config = AppStateConfig {
        db_path: temp_path.join("bench_db"),
        model_cache_path: temp_path.join("bench_models"),
    };
    let state = Arc::new(AppState::new(config).await.unwrap());
    
    let coordinator = Arc::new(
        DatabaseCoordinator::with_base_path(Some(temp_path.join("storage_db").into()))
            .unwrap()
    );
    
    let manager = McpManager::new(state, coordinator);
    (manager, temp_dir)
}

fn bench_store_log(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (manager, _temp_dir) = rt.block_on(setup_bench_manager());
    
    c.bench_function("store_log", |b| {
        b.to_async(&rt).iter(|| async {
            let log = LogEntry::new(
                LogLevel::Info,
                "bench_context".to_string(),
                "Benchmark log entry".to_string(),
                LogSource::Storage,
            );
            black_box(manager.store_log(log))
        });
    });
}

fn bench_query_logs(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (manager, _temp_dir) = rt.block_on(setup_bench_manager());
    
    // Pre-populate with logs
    rt.block_on(async {
        for i in 0..1000 {
            manager.store_log(LogEntry::new(
                LogLevel::Info,
                "context".to_string(),
                format!("Log {}", i),
                LogSource::Storage,
            )).unwrap();
        }
    });
    
    c.bench_function("query_logs_1000", |b| {
        b.to_async(&rt).iter(|| async {
            let query = LogQuery::default();
            black_box(manager.query_logs(query))
        });
    });
}

fn bench_query_logs_with_filter(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (manager, _temp_dir) = rt.block_on(setup_bench_manager());
    
    // Pre-populate with mixed logs
    rt.block_on(async {
        for i in 0..1000 {
            manager.store_log(LogEntry::new(
                if i % 2 == 0 { LogLevel::Info } else { LogLevel::Error },
                "context".to_string(),
                format!("Log {}", i),
                LogSource::Storage,
            )).unwrap();
        }
    });
    
    c.bench_function("query_logs_filtered", |b| {
        b.to_async(&rt).iter(|| async {
            let mut query = LogQuery::default();
            query.level = Some("error".to_string());
            query.limit = Some(50);
            black_box(manager.query_logs(query))
        });
    });
}

fn bench_get_log_stats(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (manager, _temp_dir) = rt.block_on(setup_bench_manager());
    
    // Pre-populate with diverse logs
    rt.block_on(async {
        for i in 0..500 {
            manager.store_log(LogEntry::new(
                if i % 3 == 0 {
                    LogLevel::Error
                } else if i % 3 == 1 {
                    LogLevel::Warn
                } else {
                    LogLevel::Info
                },
                format!("context_{}", i % 5),
                format!("Message {}", i),
                if i % 2 == 0 { LogSource::Storage } else { LogSource::Server },
            )).unwrap();
        }
    });
    
    c.bench_function("get_log_stats_500", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(manager.get_log_stats())
        });
    });
}

criterion_group!(
    benches,
    bench_store_log,
    bench_query_logs,
    bench_query_logs_with_filter,
    bench_get_log_stats
);
criterion_main!(benches);

