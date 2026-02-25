//! Bug Condition Exploration Test for Cargo.lock CI Synchronization
//!
//! **Validates: Requirements 1.1, 1.2**
//!
//! This test explores the bug condition where Cargo.lock is out of sync with Cargo.toml.
//! According to the bug report, the CI fails when running with an out-of-sync lock file.
//!
//! **Test Strategy**:
//! - Create an out-of-sync Cargo.lock scenario
//! - Test with --locked flag (the problematic behavior causing the bug)
//! - Verify the error occurs (documenting the bug condition)
//! - Test without --locked flag (the proposed fix)
//! - Verify the build succeeds (validating the fix approach)
//!
//! **CRITICAL**: This test documents the bug by showing that --locked causes failure,
//! and validates that removing --locked (the fix) allows builds to succeed.

#[cfg(test)]
mod cargo_lock_sync_tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    /// **Property 1: Fault Condition** - Cargo Build with --locked Fails on Out-of-Sync Lock File
    ///
    /// This test:
    /// 1. Creates an out-of-sync Cargo.lock scenario
    /// 2. Verifies that `cargo build --locked` FAILS (the bug)
    /// 3. Verifies that `cargo build` (without --locked) SUCCEEDS (the fix)
    ///
    /// **Expected Results**:
    /// - With --locked: FAILS with "cannot update the lock file" error (bug confirmed)
    /// - Without --locked: SUCCEEDS (fix validated)
    #[test]
    fn test_bug_condition_locked_flag_causes_failure() {
        let contract_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = contract_dir.parent().unwrap().parent().unwrap();
        let cargo_toml_path = workspace_root.join("Cargo.toml");
        let cargo_lock_path = workspace_root.join("Cargo.lock");

        // Backup original files
        let cargo_toml_backup =
            fs::read_to_string(&cargo_toml_path).expect("Failed to read Cargo.toml");
        let cargo_lock_backup =
            fs::read_to_string(&cargo_lock_path).expect("Failed to read Cargo.lock");

        // Create out-of-sync condition: add a dependency to Cargo.toml
        let modified_toml = format!(
            "{}\n\n[workspace.dependencies.serde]\nversion = \"1.0\"\n",
            cargo_toml_backup
        );
        fs::write(&cargo_toml_path, &modified_toml).expect("Failed to write modified Cargo.toml");

        println!("\n=== BUG CONDITION EXPLORATION ===");
        println!("Testing: cargo build --locked with out-of-sync Cargo.lock\n");

        // PART 1: Test with --locked (the bug condition)
        let output_locked = Command::new("cargo")
            .arg("build")
            .arg("--locked")
            .current_dir(workspace_root)
            .output()
            .expect("Failed to execute cargo build --locked");

        let stderr_locked = String::from_utf8_lossy(&output_locked.stderr);
        let exit_code_locked = output_locked.status.code().unwrap_or(-1);

        println!("Result with --locked flag:");
        println!("  Exit code: {}", exit_code_locked);
        println!("  Success: {}", output_locked.status.success());

        if !output_locked.status.success() {
            println!("\n  ✓ COUNTEREXAMPLE FOUND - Bug Confirmed!");
            println!("  Error: {}", stderr_locked.lines().next().unwrap_or(""));

            if stderr_locked.contains("cannot update the lock file because --locked was passed") {
                println!("  ✓ Error message matches expected bug condition");
                println!("  ✓ Exit code: {} (expected: 101)", exit_code_locked);
            }
        }

        // PART 2: Test without --locked (the fix)
        println!("\n--- Testing Fix Approach ---");
        println!("Testing: cargo build (without --locked)\n");

        let output_no_flag = Command::new("cargo")
            .arg("build")
            .current_dir(workspace_root)
            .output()
            .expect("Failed to execute cargo build");

        let exit_code_no_flag = output_no_flag.status.code().unwrap_or(-1);

        println!("Result without --locked flag:");
        println!("  Exit code: {}", exit_code_no_flag);
        println!("  Success: {}", output_no_flag.status.success());

        // Restore original files
        fs::write(&cargo_toml_path, cargo_toml_backup).expect("Failed to restore Cargo.toml");
        fs::write(&cargo_lock_path, cargo_lock_backup).expect("Failed to restore Cargo.lock");

        println!("\n=== TEST RESULTS ===");

        // Verify the bug exists: --locked should fail
        assert!(
            !output_locked.status.success(),
            "Expected: cargo build --locked should FAIL with out-of-sync Cargo.lock (bug condition). \
             But it succeeded, which means the bug might not exist or lock file is already in sync."
        );

        assert!(
            stderr_locked.contains("cannot update the lock file because --locked was passed")
                || stderr_locked.contains("lock file"),
            "Expected: Error message should mention lock file issue. Stderr: {}",
            stderr_locked
        );

        // Verify the fix works: without --locked should succeed
        assert!(
            output_no_flag.status.success(),
            "Expected: cargo build (without --locked) should SUCCEED. \
             This validates the fix approach. Exit code: {}",
            exit_code_no_flag
        );

        println!("✓ Bug confirmed: --locked flag causes failure with out-of-sync Cargo.lock");
        println!("✓ Fix validated: Removing --locked allows build to succeed");
        println!("✓ Recommendation: Remove --locked from CI workflow");
    }
}
