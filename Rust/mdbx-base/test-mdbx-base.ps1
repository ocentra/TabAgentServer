# MDBX Test Runner - ONE SCRIPT TO RULE THEM ALL
# Usage:
#   .\test.ps1              - Run ALL tests
#   .\test.ps1 debug        - Run DEBUG suite (find root cause)
#   .\test.ps1 tdd          - Run TDD suite (systematic learning)
#   .\test.ps1 <test_name>  - Run specific test

cd "E:\Desktop\TabAgent\TabAgentServer\Rust\mdbx-base"

$mode = $args[0]

if ($mode -eq "debug") {
    # Debug Suite - Find root cause of errors
    Write-Host "`n[DEBUG] MDBX DEBUG SUITE - Finding Root Cause`n" -ForegroundColor Red
    
    Write-Host "[1] Check Temp Directories" -ForegroundColor Yellow
    cargo test test_check_temp_directories -- --nocapture --test-threads=1
    
    Write-Host "`n[2] Debug Error Codes" -ForegroundColor Yellow
    cargo test test_debug_error_codes -- --nocapture --test-threads=1
    
    Write-Host "`n[OK] DEBUG SUITE COMPLETE - Check output above!`n" -ForegroundColor Green
    
} elseif ($mode -eq "tdd") {
    # TDD Suite - Systematic MDBX learning
    Write-Host "`n[TDD] MDBX TDD SUITE - Learning MDBX Systematically`n" -ForegroundColor Cyan
    
    Write-Host "[1] Multiple Named Databases" -ForegroundColor Yellow
    cargo test test_multiple_named_databases -- --nocapture --test-threads=1
    
    Write-Host "`n[2] Only Named Databases (Positive Test)" -ForegroundColor Green
    cargo test test_mdbx_only_named_databases -- --nocapture --test-threads=1
    
    Write-Host "`n[3] Mixing Unnamed + Named (Negative Test)" -ForegroundColor Red
    cargo test test_mdbx_unnamed_and_named_should_fail -- --nocapture --test-threads=1
    
    Write-Host "`n[4] Reopen Existing Database" -ForegroundColor Yellow
    cargo test test_mdbx_reopen_existing_database -- --nocapture --test-threads=1
    
    Write-Host "`n[5] Stale Files Handling" -ForegroundColor Yellow
    cargo test test_mdbx_stale_files_handling -- --nocapture --test-threads=1
    
    Write-Host "`n[OK] TDD SUITE COMPLETE`n" -ForegroundColor Green
    
} elseif ($mode) {
    # Run specific test
    Write-Host "`n[RUN] Running: $mode`n" -ForegroundColor Yellow
    cargo test "$mode" -- --nocapture --test-threads=1
    
} else {
    # Run ALL tests
    Write-Host "`n[ALL] RUNNING ALL MDBX-BASE TESTS`n" -ForegroundColor Cyan
    cargo test -p mdbx-base -- --nocapture --test-threads=1
}

