# Cargo.lock CI Error Fix Design

## Overview

The CI workflow fails when running `cargo build --locked` because the Cargo.lock file is out of sync with Cargo.toml dependencies. The `--locked` flag prevents Cargo from updating the lock file, causing the build to fail with exit code 101. The fix involves removing the `--locked` flag from the CI workflow to allow Cargo to update the lock file when needed, or alternatively using `--frozen` for offline builds. This is a minimal change that resolves the synchronization issue while maintaining all existing CI validation checks.

## Glossary

- **Bug_Condition (C)**: The condition that triggers the bug - when Cargo.lock is out of sync with Cargo.toml and the `--locked` flag prevents updates
- **Property (P)**: The desired behavior - cargo build should succeed by either updating Cargo.lock or using an appropriate flag
- **Preservation**: Existing CI checks (format, test, clippy) and trigger conditions that must remain unchanged
- **Cargo.lock**: Lock file that records exact versions of dependencies for reproducible builds
- **--locked flag**: Cargo flag that prevents any updates to Cargo.lock, causing failure if the file is out of sync
- **--frozen flag**: Cargo flag that allows offline builds without network access but doesn't prevent lock file reads

## Bug Details

### Fault Condition

The bug manifests when the CI workflow executes `cargo build --locked` and the Cargo.lock file is out of sync with the dependencies declared in Cargo.toml. The `--locked` flag explicitly prevents Cargo from updating the lock file, causing the build to fail immediately.

**Formal Specification:**
```
FUNCTION isBugCondition(input)
  INPUT: input of type CIBuildExecution
  OUTPUT: boolean
  
  RETURN input.cargoCommand CONTAINS "--locked"
         AND input.cargoLockFile.isOutOfSync(input.cargoTomlFile)
         AND input.buildStep = "cargo build"
END FUNCTION
```

### Examples

- **Example 1**: Developer updates a dependency version in Cargo.toml but doesn't commit the updated Cargo.lock. CI runs `cargo build --locked` and fails with "cannot update the lock file because --locked was passed to prevent this"
- **Example 2**: A transitive dependency is updated upstream, causing Cargo.lock to be out of sync. CI build fails with exit code 101
- **Example 3**: New dependency is added to Cargo.toml locally but Cargo.lock is not regenerated before pushing. CI fails on the build step
- **Edge case**: Cargo.lock is in sync - build succeeds normally (this is the preservation case)

## Expected Behavior

### Preservation Requirements

**Unchanged Behaviors:**
- Cargo format check must continue to validate code formatting
- Cargo test must continue to run all tests
- Cargo clippy must continue to perform linting with warnings as errors
- CI workflow triggers (pull requests and pushes to main affecting contract/** or .github/workflows/**) must remain unchanged
- Cargo caching strategy must remain unchanged
- Working directory (contract/) must remain unchanged

**Scope:**
All CI workflow steps that do NOT involve the cargo build command should be completely unaffected by this fix. This includes:
- Checkout and Rust toolchain setup steps
- Cargo registry caching configuration
- Format, test, and clippy validation steps
- Workflow trigger conditions and job configuration

## Hypothesized Root Cause

Based on the bug description, the most likely issues are:

1. **Incorrect Flag Usage**: The `--locked` flag is currently present in the CI workflow (or implied by default behavior), which is too strict for a CI environment where lock file updates should be allowed
   - The flag prevents any lock file modifications
   - This is appropriate for production deployments but not for CI validation

2. **Missing Flag in Workflow**: The workflow file doesn't explicitly specify a cargo build flag, and the default behavior may be causing issues
   - Need to verify if `--locked` is explicitly used or if it's default behavior
   - May need to explicitly use `--frozen` or remove restrictive flags

3. **Lock File Not Committed**: Developers may not be committing updated Cargo.lock files after dependency changes
   - This is a workflow issue but the CI should handle it gracefully
   - The fix should allow CI to update the lock file or use appropriate flags

4. **Cargo Version Mismatch**: Different Cargo versions between local development and CI may handle lock files differently
   - Less likely but possible if toolchain versions differ

## Correctness Properties

Property 1: Fault Condition - Cargo Build Succeeds with Out-of-Sync Lock File

_For any_ CI build execution where the Cargo.lock file is out of sync with Cargo.toml dependencies, the fixed cargo build command SHALL successfully build the contract by either updating the lock file or using an appropriate flag that allows the build to proceed.

**Validates: Requirements 2.1, 2.2**

Property 2: Preservation - Existing CI Checks Continue to Function

_For any_ CI workflow execution that does NOT involve the cargo build step (formatting, testing, clippy checks), the fixed workflow SHALL produce exactly the same behavior as the original workflow, preserving all existing validation checks and their configurations.

**Validates: Requirements 3.1, 3.2, 3.3**

## Fix Implementation

### Changes Required

Assuming our root cause analysis is correct:

**File**: `.github/workflows/contract-ci.yml`

**Step**: `Cargo build`

**Specific Changes**:
1. **Remove --locked flag if present**: Check if the cargo build command explicitly uses `--locked` and remove it
   - Current command: `cargo build` (no flag visible, but may be implied)
   - Fixed command: `cargo build` (explicitly without --locked)

2. **Alternative: Use --frozen flag**: If reproducibility is desired without network access
   - Fixed command: `cargo build --frozen`
   - This allows reading the lock file but doesn't require it to be perfectly in sync

3. **Verify no implicit --locked behavior**: Ensure the Rust toolchain setup doesn't set --locked as default
   - Check actions-rs/toolchain@v1 configuration
   - Verify no environment variables enforce --locked

4. **Document the approach**: Add a comment explaining why --locked is not used in CI
   - Clarify that CI should allow lock file updates
   - Note that production builds may use --locked for reproducibility

5. **Consider adding lock file validation**: Optionally add a step to check if Cargo.lock needs updating and fail with a clear message
   - This would be a separate step before the build
   - Helps developers understand when they need to commit lock file changes

## Testing Strategy

### Validation Approach

The testing strategy follows a two-phase approach: first, surface counterexamples that demonstrate the bug on unfixed code, then verify the fix works correctly and preserves existing behavior.

### Exploratory Fault Condition Checking

**Goal**: Surface counterexamples that demonstrate the bug BEFORE implementing the fix. Confirm or refute the root cause analysis. If we refute, we will need to re-hypothesize.

**Test Plan**: Manually create an out-of-sync Cargo.lock scenario and run the CI workflow on the UNFIXED code to observe the failure. This confirms the bug condition and validates our understanding of the root cause.

**Test Cases**:
1. **Out-of-Sync Lock File Test**: Modify Cargo.toml to add a dependency, don't update Cargo.lock, push to trigger CI (will fail on unfixed code)
2. **Version Mismatch Test**: Change a dependency version in Cargo.toml without updating lock file (will fail on unfixed code)
3. **Transitive Dependency Test**: Force a transitive dependency update scenario (will fail on unfixed code)
4. **In-Sync Test**: Ensure Cargo.lock is in sync and verify build succeeds (should pass on unfixed code - this is preservation)

**Expected Counterexamples**:
- CI build fails with "cannot update the lock file because --locked was passed to prevent this"
- Possible causes: --locked flag is present (explicitly or implicitly), preventing lock file updates

### Fix Checking

**Goal**: Verify that for all inputs where the bug condition holds, the fixed function produces the expected behavior.

**Pseudocode:**
```
FOR ALL input WHERE isBugCondition(input) DO
  result := cargoBuild_fixed(input)
  ASSERT result.buildSucceeds = true
  ASSERT result.exitCode = 0
END FOR
```

### Preservation Checking

**Goal**: Verify that for all inputs where the bug condition does NOT hold, the fixed function produces the same result as the original function.

**Pseudocode:**
```
FOR ALL input WHERE NOT isBugCondition(input) DO
  ASSERT cargoBuild_original(input) = cargoBuild_fixed(input)
END FOR
```

**Testing Approach**: Property-based testing is recommended for preservation checking because:
- It generates many test cases automatically across the input domain
- It catches edge cases that manual unit tests might miss
- It provides strong guarantees that behavior is unchanged for all non-buggy inputs

**Test Plan**: Observe behavior on UNFIXED code first for in-sync lock files and other CI steps, then write property-based tests capturing that behavior.

**Test Cases**:
1. **In-Sync Build Preservation**: Observe that builds with synchronized Cargo.lock succeed on unfixed code, then verify this continues after fix
2. **Format Check Preservation**: Observe that cargo fmt check works correctly on unfixed code, then verify this continues after fix
3. **Test Execution Preservation**: Observe that cargo test runs correctly on unfixed code, then verify this continues after fix
4. **Clippy Check Preservation**: Observe that cargo clippy runs correctly on unfixed code, then verify this continues after fix

### Unit Tests

- Test cargo build with out-of-sync Cargo.lock (should succeed after fix)
- Test cargo build with in-sync Cargo.lock (should continue to succeed)
- Test that all other CI steps (format, test, clippy) continue to execute
- Test edge case where Cargo.toml has syntax errors (should fail appropriately)

### Property-Based Tests

- Generate random dependency configurations and verify builds succeed regardless of lock file sync status
- Generate random CI workflow configurations and verify preservation of non-build steps
- Test across many scenarios with different lock file states

### Integration Tests

- Test full CI workflow with intentionally out-of-sync lock file
- Test CI workflow with in-sync lock file to verify no regression
- Test that PR and push triggers continue to work correctly
- Test that caching behavior remains unchanged
