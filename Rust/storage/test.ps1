# Storage Test Runner - COMPREHENSIVE
cd "E:\Desktop\TabAgent\TabAgentServer\Rust"

Write-Host "`nSTORAGE TEST SUITE - RUNNING ALL TESTS`n" -ForegroundColor Cyan

Write-Host "UNIT TESTS (src/lib.rs)..." -ForegroundColor Yellow
cargo test --package storage --lib -- --nocapture --test-threads=1 --show-output

Write-Host "`nINTEGRATION TESTS (tests/*.rs)..." -ForegroundColor Yellow
cargo test --package storage --test '*' -- --nocapture --test-threads=1 --show-output

Write-Host "`nSTORAGE TEST SUITE COMPLETE`n" -ForegroundColor Green
