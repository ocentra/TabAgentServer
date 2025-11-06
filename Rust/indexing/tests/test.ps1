# Indexing Test Runner - ONE SCRIPT FOR ALL TESTS
# Usage:
#   .\test.ps1                - Run ALL tests
#   .\test.ps1 graph          - Run graph integration tests
#   .\test.ps1 vector         - Run vector integration tests
#   .\test.ps1 structural     - Run structural integration tests
#   .\test.ps1 integration    - Run all integration tests
#   .\test.ps1 unit           - Run all unit tests
#   .\test.ps1 stress         - Run stress tests (SLOW!)
#   .\test.ps1 <test_name>    - Run specific test

cd "E:\Desktop\TabAgent\TabAgentServer\Rust"

$mode = $args[0]
$timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$reportFile = "E:\Desktop\TabAgent\TabAgentServer\Rust\indexing\tests\TEST_REPORT_$timestamp.md"
$reportPath = "E:\Desktop\TabAgent\TabAgentServer\Rust\indexing\tests\TEST_REPORT.md"
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
$startTime = Get-Date

$testOutput = switch ($mode) {
    "graph" {
        Write-Host "`n[GRAPH] GRAPH INTEGRATION TESTS`n" -ForegroundColor Cyan
        cargo test --package indexing --test mod integration::graph_tests -- --nocapture --test-threads=1 2>&1
    }
    "vector" {
        Write-Host "`n[VECTOR] VECTOR INTEGRATION TESTS`n" -ForegroundColor Cyan
        cargo test --package indexing --test mod integration::vector_tests -- --nocapture --test-threads=1 2>&1
    }
    "structural" {
        Write-Host "`n[STRUCT] STRUCTURAL INTEGRATION TESTS`n" -ForegroundColor Cyan
        cargo test --package indexing --test mod integration::structural_tests -- --nocapture --test-threads=1 2>&1
    }
    "integration" {
        Write-Host "`n[INT] ALL INTEGRATION TESTS`n" -ForegroundColor Cyan
        cargo test --package indexing --test mod integration -- --nocapture --test-threads=1 2>&1
    }
    "unit" {
        Write-Host "`n[UNIT] ALL UNIT TESTS`n" -ForegroundColor Cyan
        cargo test --package indexing --test mod unit -- --nocapture --test-threads=1 2>&1
    }
    "stress" {
        Write-Host "`n[STRESS] STRESS TESTS (Concurrency)`n" -ForegroundColor Yellow
        cargo test --package indexing --test mod stress -- --nocapture 2>&1
    }
    $null {
        Write-Host "`n[ALL] RUNNING ALL INDEXING TESTS`n" -ForegroundColor Cyan
        cargo test --package indexing --test mod --no-fail-fast -- --nocapture 2>&1
    }
    default {
        Write-Host "`n[RUN] Running: $mode`n" -ForegroundColor Yellow
        cargo test --package indexing --test mod "$mode" -- --nocapture --test-threads=1 2>&1
    }
}

# Output to console
$testOutput | Tee-Object -Variable capturedOutput

# Generate MD report
$passed = ($capturedOutput | Select-String "test result: ok" | Measure-Object).Count
$failed = ($capturedOutput | Select-String "FAILED" | Measure-Object).Count
$total = ($capturedOutput | Select-String "running \d+ test" | ForEach-Object { if ($_ -match "running (\d+) test") { $matches[1] } })

@"
# Indexing Crate Test Report
**Generated**: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
**Mode**: $mode
**Total Tests**: $total
**Passed**: $passed
**Failed**: $failed

## Test Output
``````
$capturedOutput
``````
"@ | Out-File -FilePath $reportFile -Encoding UTF8

Write-Host "`n[REPORT] Test report saved to: $reportFile" -ForegroundColor Green

