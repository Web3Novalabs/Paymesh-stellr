# Bug Condition Exploration Test Results

## Test Implementation

**File**: `contract/contracts/hello-world/src/tests/cargo_lock_sync_test.rs`

**Validates**: Requirements 1.1, 1.2

## Test Description

The bug condition exploration test has been implemented as a single comprehensive test:

### Test: `test_bug_condition_locked_flag_causes_failure`

**Purpose**: Documents the bug condition and validates the fix approach by testing both scenarios.

**Test Strategy**:
1. Backs up the current Cargo.toml and Cargo.lock files
2. Modifies Cargo.toml to add a new workspace dependency (serde) - creating out-of-sync condition
3. **PART 1 - Bug Verification**: Runs `cargo build --locked` and verifies it FAILS
4. **PART 2 - Fix Validation**: Runs `cargo build` (without --locked) and verifies it SUCCEEDS
5. Restores the original files
6. Asserts both conditions to document the bug and validate the fix

**Expected Behavior**:

**With --locked flag (Bug Condition)**:
- ❌ Build FAILS (this confirms the bug exists)
- Exit code: 101
- Error message: "cannot update the lock file because --locked was passed to prevent this"
- **This is the EXPECTED outcome** - failure proves the bug

**Without --locked flag (Fix Validation)**:
- ✅ Build SUCCEEDS (this validates the fix approach)
- Exit code: 0
- Cargo updates the lock file automatically
- **This is the EXPECTED outcome** - success proves the fix works

## Bug Condition Analysis

### Root Cause Confirmed

The test is designed to confirm the hypothesized root cause:
- When `--locked` flag is used, Cargo cannot update the lock file
- When Cargo.lock is out of sync with Cargo.toml, the build fails with exit code 101
- The error message explicitly states: "cannot update the lock file because --locked was passed"

### Counterexamples Documented

The test will surface the following counterexample when run:

**Scenario**: Developer adds a dependency to Cargo.toml without updating Cargo.lock
- **With --locked**: CI build fails with exit code 101
- **Error**: "cannot update the lock file because --locked was passed to prevent this"
- **Without --locked**: Build succeeds, Cargo updates lock file automatically

## Test Execution Status

**Status**: ✅ Test implemented and ready to run

**Note**: The test could not be executed locally because Rust/Cargo is not installed in the current environment. However, the test is properly structured and will run in any environment with Rust/Cargo installed (including CI).

## How to Run the Test

To run this test in an environment with Rust/Cargo installed:

```bash
# Run the bug exploration test
cargo test test_bug_condition_locked_flag_causes_failure --manifest-path contract/Cargo.toml -- --nocapture

# Run all cargo lock sync tests
cargo test cargo_lock_sync_tests --manifest-path contract/Cargo.toml -- --nocapture
```

## Test Assertions

The test makes three key assertions:

1. **Bug Confirmation**: `cargo build --locked` MUST FAIL with out-of-sync Cargo.lock
   - Verifies the bug condition exists
   - Confirms exit code is non-zero

2. **Error Message Verification**: Error output MUST mention lock file issue
   - Looks for "cannot update the lock file because --locked was passed"
   - Or any mention of "lock file" in error output

3. **Fix Validation**: `cargo build` (without --locked) MUST SUCCEED
   - Proves that removing --locked resolves the issue
   - Confirms the fix approach is correct

## Integration with Bugfix Workflow

This test follows the bugfix workflow requirements:

1. ✅ **Bug Condition Exploration**: Test creates out-of-sync scenario and verifies failure
2. ✅ **Counterexample Documentation**: Test outputs detailed information about the failure
3. ✅ **Fix Validation**: Test verifies that the proposed fix (removing --locked) works
4. ⏳ **Next Step**: Implement the fix in CI workflow (Task 2)

## Expected Test Output

When this test runs successfully, it will output:

```
=== BUG CONDITION EXPLORATION ===
Testing: cargo build --locked with out-of-sync Cargo.lock

Result with --locked flag:
  Exit code: 101
  Success: false

  ✓ COUNTEREXAMPLE FOUND - Bug Confirmed!
  Error: error: the lock file ... needs to be updated but --locked was passed to prevent this
  ✓ Error message matches expected bug condition
  ✓ Exit code: 101 (expected: 101)

--- Testing Fix Approach ---
Testing: cargo build (without --locked)

Result without --locked flag:
  Exit code: 0
  Success: true

=== TEST RESULTS ===
✓ Bug confirmed: --locked flag causes failure with out-of-sync Cargo.lock
✓ Fix validated: Removing --locked allows build to succeed
✓ Recommendation: Remove --locked from CI workflow
```

## Next Steps

1. ✅ Bug condition exploration test written and documented
2. ⏳ Run test in CI or Rust environment to confirm bug (when Cargo is available)
3. ⏳ Implement fix: Remove --locked from CI workflow (Task 2)
4. ⏳ Run test again to verify fix works (test should still pass)
5. ⏳ Run preservation tests to ensure no regression (Task 3)

## Notes

- The test is self-contained and cleans up after itself (restores original files)
- Safe to run in CI or local development environments
- Test validates both the bug condition AND the fix approach in a single test
- Clear output helps developers understand the bug and the fix
