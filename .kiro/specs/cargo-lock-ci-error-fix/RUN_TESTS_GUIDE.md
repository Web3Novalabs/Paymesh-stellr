# Test Execution Guide - Cargo.lock CI Error Fix

## Quick Start

Run all tests with a single command:

```bash
cd contract
cargo test --nocapture
```

## Individual Test Commands

### 1. Bug Condition Exploration Test

This test verifies the bug exists and validates the fix approach:

```bash
cd contract
cargo test test_bug_condition_locked_flag_causes_failure --nocapture
```

**What this test does:**
- Creates an out-of-sync Cargo.lock scenario
- Verifies `cargo build --locked` FAILS (confirms bug)
- Verifies `cargo build` (without --locked) SUCCEEDS (validates fix)

**Expected output:**
```
=== BUG CONDITION EXPLORATION ===
✓ Bug confirmed: --locked flag causes failure
✓ Fix validated: Removing --locked allows build to succeed
```

### 2. Preservation Property Tests

These tests verify no regressions were introduced:

```bash
cd contract
cargo test cargo_lock_preservation_test --nocapture
```

**What these tests do:**
- Verify cargo build still works with in-sync Cargo.lock
- Verify cargo format, test, and clippy continue to work
- Use property-based testing to test multiple scenarios

**Expected output:**
```
test cargo_lock_preservation_test::test_cargo_build_succeeds_with_synced_lock ... ok
test cargo_lock_preservation_test::test_cargo_format_check_continues ... ok
test cargo_lock_preservation_test::test_cargo_test_continues ... ok
test cargo_lock_preservation_test::test_cargo_clippy_continues ... ok
test cargo_lock_preservation_test::prop_build_succeeds_across_runs ... ok
test cargo_lock_preservation_test::prop_all_ci_steps_execute ... ok
test cargo_lock_preservation_test::prop_cargo_cache_behavior_preserved ... ok
test cargo_lock_preservation_test::prop_working_directory_preserved ... ok
test cargo_lock_preservation_test::prop_ci_trigger_conditions_preserved ... ok
```

### 3. Run All Tests (Recommended)

Run everything at once:

```bash
cd contract
cargo test --nocapture
```

This will run:
- Bug condition exploration test (1 test)
- Preservation property tests (9 tests)
- All other existing tests in the contract

## Alternative: Using Provided Scripts

### Linux/Mac:
```bash
bash .kiro/specs/cargo-lock-ci-error-fix/run_preservation_tests.sh
```

### Windows:
```powershell
powershell .kiro/specs/cargo-lock-ci-error-fix/run_preservation_tests.ps1
```

## Test Execution Checklist

After running the tests, verify:

- [ ] Bug condition test passes (both parts)
  - [ ] PART 1: `--locked` flag causes failure ✓
  - [ ] PART 2: Without `--locked` succeeds ✓

- [ ] All 9 preservation tests pass
  - [ ] `test_cargo_build_succeeds_with_synced_lock` ✓
  - [ ] `test_cargo_format_check_continues` ✓
  - [ ] `test_cargo_test_continues` ✓
  - [ ] `test_cargo_clippy_continues` ✓
  - [ ] `prop_build_succeeds_across_runs` ✓
  - [ ] `prop_all_ci_steps_execute` ✓
  - [ ] `prop_cargo_cache_behavior_preserved` ✓
  - [ ] `prop_working_directory_preserved` ✓
  - [ ] `prop_ci_trigger_conditions_preserved` ✓

- [ ] No test failures or errors
- [ ] All tests show "ok" status

## Troubleshooting

### If tests fail to compile:
```bash
cd contract
cargo check
```

### If you get "cargo: command not found":
Install Rust from https://rustup.rs/

### If tests fail:
1. Check the error message carefully
2. Verify Cargo.lock is in sync: `cargo update --dry-run`
3. Try running tests individually to isolate the issue

## CI Verification

The tests will also run automatically in CI when you push changes. Check the GitHub Actions workflow:

1. Go to your repository on GitHub
2. Click "Actions" tab
3. Find the "Contract CI" workflow
4. Verify all steps pass, especially "Cargo build"

## Expected Timeline

- **Bug condition test**: ~30 seconds (creates out-of-sync scenario)
- **Preservation tests**: ~1-2 minutes (property-based tests run multiple cases)
- **Total**: ~2-3 minutes

## Success Criteria

Task 4 checkpoint is complete when:

✅ All 10 tests pass (1 bug condition + 9 preservation)
✅ No test failures or errors
✅ CI workflow runs successfully

## Next Steps After Tests Pass

1. Mark Task 4 as complete
2. Commit all changes
3. Push to trigger CI
4. Verify CI passes
5. Bugfix is complete! 🎉

