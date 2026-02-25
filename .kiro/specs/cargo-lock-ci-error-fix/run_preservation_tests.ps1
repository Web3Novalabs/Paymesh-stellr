# Script to run preservation property tests for Cargo.lock CI Error Fix
# This script should be run from the repository root

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Running Preservation Property Tests" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "These tests verify that existing CI checks continue to function" -ForegroundColor Yellow
Write-Host "correctly for non-buggy inputs (in-sync Cargo.lock scenarios)." -ForegroundColor Yellow
Write-Host ""
Write-Host "EXPECTED OUTCOME: Tests should PASS on unfixed code" -ForegroundColor Green
Write-Host "This confirms the baseline behavior that must be preserved." -ForegroundColor Green
Write-Host ""

Set-Location contract

Write-Host "Running preservation tests..." -ForegroundColor Cyan
Write-Host ""

cargo test --package hello-world cargo_lock_preservation_test -- --nocapture

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Preservation Tests Complete" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "If all tests passed, the baseline behavior has been confirmed." -ForegroundColor Green
Write-Host "This behavior must be preserved after implementing the fix." -ForegroundColor Green
