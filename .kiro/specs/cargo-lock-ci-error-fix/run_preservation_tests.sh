#!/bin/bash

# Script to run preservation property tests for Cargo.lock CI Error Fix
# This script should be run from the repository root

set -e

echo "=========================================="
echo "Running Preservation Property Tests"
echo "=========================================="
echo ""
echo "These tests verify that existing CI checks continue to function"
echo "correctly for non-buggy inputs (in-sync Cargo.lock scenarios)."
echo ""
echo "EXPECTED OUTCOME: Tests should PASS on unfixed code"
echo "This confirms the baseline behavior that must be preserved."
echo ""

cd contract

echo "Running preservation tests..."
echo ""

cargo test --package hello-world cargo_lock_preservation_test -- --nocapture

echo ""
echo "=========================================="
echo "Preservation Tests Complete"
echo "=========================================="
echo ""
echo "If all tests passed, the baseline behavior has been confirmed."
echo "This behavior must be preserved after implementing the fix."
