# Implementation Plan

- [x] 1. Write bug condition exploration test
  - **Property 1: Fault Condition** - Cargo Build Fails with Out-of-Sync Lock File
  - **CRITICAL**: This test MUST FAIL on unfixed code - failure confirms the bug exists
  - **DO NOT attempt to fix the test or the code when it fails**
  - **NOTE**: This test encodes the expected behavior - it will validate the fix when it passes after implementation
  - **GOAL**: Surface counterexamples that demonstrate the bug exists
  - **Scoped PBT Approach**: Create an out-of-sync Cargo.lock scenario and verify CI build fails with the specific error
  - Test implementation: Modify Cargo.toml to add/change a dependency without updating Cargo.lock, trigger CI workflow
  - The test assertions should verify: build fails with "cannot update the lock file because --locked was passed to prevent this" error and exit code 101
  - Run test on UNFIXED code (current CI workflow with --locked flag or restrictive behavior)
  - **EXPECTED OUTCOME**: Test FAILS (this is correct - it proves the bug exists)
  - Document counterexamples found: specific error messages, exit codes, and conditions that trigger the failure
  - Mark task complete when test is written, run, and failure is documented
  - _Requirements: 1.1, 1.2_

- [x] 2. Write preservation property tests (BEFORE implementing fix)
  - **Property 2: Preservation** - Existing CI Checks Continue to Function
  - **IMPORTANT**: Follow observation-first methodology
  - Observe behavior on UNFIXED code for non-buggy inputs (in-sync Cargo.lock scenarios)
  - Write property-based tests capturing observed behavior patterns:
    - Cargo build succeeds when Cargo.lock is in sync with Cargo.toml
    - Cargo format check continues to validate code formatting
    - Cargo test continues to run all tests
    - Cargo clippy continues to perform linting with warnings as errors
    - CI workflow triggers (pull requests and pushes to main) continue to work
  - Property-based testing generates many test cases for stronger guarantees
  - Run tests on UNFIXED code
  - **EXPECTED OUTCOME**: Tests PASS (this confirms baseline behavior to preserve)
  - Mark task complete when tests are written, run, and passing on unfixed code
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 3. Fix for Cargo.lock CI synchronization error

  - [x] 3.1 Implement the fix in CI workflow
    - Read `.github/workflows/contract-ci.yml` to identify the cargo build step
    - Check if `--locked` flag is explicitly present in the cargo build command
    - Remove `--locked` flag if present, or ensure cargo build runs without restrictive flags
    - Alternative: Replace with `--frozen` flag if offline builds are desired
    - Add comment explaining why --locked is not used in CI (allows lock file updates)
    - Verify no implicit --locked behavior from toolchain configuration
    - _Bug_Condition: isBugCondition(input) where input.cargoCommand CONTAINS "--locked" AND input.cargoLockFile.isOutOfSync(input.cargoTomlFile)_
    - _Expected_Behavior: cargo build succeeds by updating lock file or using appropriate flag (Requirements 2.1, 2.2)_
    - _Preservation: All CI steps except cargo build remain unchanged (Requirements 3.1, 3.2, 3.3)_
    - _Requirements: 1.1, 1.2, 2.1, 2.2, 3.1, 3.2, 3.3_

  - [x] 3.2 Verify bug condition exploration test now passes
    - **Property 1: Expected Behavior** - Cargo Build Succeeds with Out-of-Sync Lock File
    - **IMPORTANT**: Re-run the SAME test from task 1 - do NOT write a new test
    - The test from task 1 encodes the expected behavior
    - When this test passes, it confirms the expected behavior is satisfied
    - Run bug condition exploration test from step 1 (create out-of-sync scenario and trigger CI)
    - **EXPECTED OUTCOME**: Test PASSES (confirms bug is fixed - build succeeds without lock file error)
    - _Requirements: 2.1, 2.2_

  - [x] 3.3 Verify preservation tests still pass
    - **Property 2: Preservation** - Existing CI Checks Continue to Function
    - **IMPORTANT**: Re-run the SAME tests from task 2 - do NOT write new tests
    - Run preservation property tests from step 2
    - **EXPECTED OUTCOME**: Tests PASS (confirms no regressions)
    - Confirm all tests still pass after fix: in-sync builds, format checks, test execution, clippy checks, workflow triggers

- [x] 4. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
