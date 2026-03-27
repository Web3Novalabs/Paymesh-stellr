# Next Steps for Issue #168 - Group Name Validation Tests

## What Has Been Done ✅

1. **Created comprehensive test file**: `src/tests/group_name_validation_test.rs`
   - 17 test cases covering all requirements from the issue
   - Tests for both `create_autoshare` and `update_group_name` functions

2. **Registered test module**: Updated `src/lib.rs` to include the new test module

3. **Created documentation**: `GROUP_NAME_VALIDATION_TESTS.md` explaining all tests

## What You Need to Do Next

### Step 1: Install Dependencies

You need to install Rust and Stellar CLI to run the tests:

1. **Install Rust**:
   ```bash
   # Download and run rustup installer from:
   https://rustup.rs/
   
   # Or use Windows installer:
   https://win.rustup.rs/
   ```

2. **Install Stellar CLI**:
   ```bash
   cargo install --locked stellar-cli
   ```

### Step 2: Run the Tests

Once dependencies are installed:

```bash
cd contract/contracts/hello-world
cargo test group_name_validation_test
```

Expected output: All tests should pass or fail according to the validation logic.

### Step 3: Review Test Results

Some tests are marked with `#[should_panic]` because they expect errors:
- Empty names
- Whitespace-only names
- Names exceeding 60 characters
- Updates on deactivated groups

### Step 4: Commit Your Changes

```bash
git add contract/contracts/hello-world/src/tests/group_name_validation_test.rs
git add contract/contracts/hello-world/src/lib.rs
git add contract/contracts/hello-world/GROUP_NAME_VALIDATION_TESTS.md
git commit -m "Add comprehensive group name validation tests for issue #168"
```

### Step 5: Push to Your Branch

```bash
git push origin issue#168
```

### Step 6: Create Pull Request

1. Go to the GitHub repository
2. Create a Pull Request from your `issue#168` branch
3. Reference issue #168 in the PR description
4. Include test results in the PR description

## Test Coverage Summary

The test suite covers all 11 requirements from the issue:

1. ✅ Name at exact minimum length (1 char)
2. ✅ Name at exact maximum length (60 chars)
3. ✅ Name exceeding max length (61 chars) returns error
4. ✅ Empty string returns Error::EmptyName
5. ✅ Whitespace-only string returns error
6. ✅ Name with leading/trailing spaces (documents behavior)
7. ✅ Name with special characters (emoji, unicode)
8. ✅ Name with only numbers
9. ✅ update_group_name with same name as current
10. ✅ update_group_name on deactivated group
11. ✅ Name with newlines or tab characters

## Files Modified/Created

```
contract/contracts/hello-world/
├── src/
│   ├── lib.rs (modified - added test module)
│   └── tests/
│       └── group_name_validation_test.rs (new - 17 test cases)
├── GROUP_NAME_VALIDATION_TESTS.md (new - documentation)
└── NEXT_STEPS.md (this file)
```

## Troubleshooting

### If tests fail:

1. Check that the validation logic in `autoshare_logic.rs` matches expectations
2. Review the `is_valid_name` function behavior
3. Verify that `Error::EmptyName` is returned for invalid names

### If cargo is not found:

1. Restart your terminal after installing Rust
2. Verify installation: `cargo --version`
3. Add Rust to PATH if needed

## Questions?

If you encounter any issues:
1. Check the test documentation in `GROUP_NAME_VALIDATION_TESTS.md`
2. Review existing tests in `src/tests/autoshare_test.rs` for patterns
3. Consult the Soroban documentation: https://soroban.stellar.org/

## Success Criteria

Your contribution is complete when:
- ✅ All 17 tests are implemented
- ✅ Tests run successfully with `cargo test`
- ✅ Code is committed to your branch
- ✅ Pull request is created and references issue #168
- ✅ Tests demonstrate the validation behavior clearly
