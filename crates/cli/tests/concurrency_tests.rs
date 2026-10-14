use pe_cli::SingleFlightLock;
use pe_core::TerminalStatus;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_concurrent_lock_rejection_and_release() {
    let dir = tempdir().unwrap();
    let lock_path = dir.path().join("concurrency_test.lock");

    let lock1 = SingleFlightLock::acquire(Some(&lock_path)).expect("First acquisition succeeds");

    // Second acquisition must return Busy
    let lock2_err =
        SingleFlightLock::acquire(Some(&lock_path)).expect_err("Second acquisition fails");
    assert_eq!(lock2_err, TerminalStatus::Busy);

    // Release first lock
    drop(lock1);

    // Now acquisition succeeds
    let lock3 = SingleFlightLock::acquire(Some(&lock_path)).expect("Re-acquisition succeeds");
    drop(lock3);
}

#[test]
fn test_dead_pid_stale_lock_recovery() {
    let dir = tempdir().unwrap();
    let lock_path = dir.path().join("stale_test.lock");

    // Artificially write a PID that is guaranteed not to exist
    fs::write(&lock_path, "9999999\n").unwrap();

    // Lock acquisition must clean up dead PID and succeed
    let lock = SingleFlightLock::acquire(Some(&lock_path)).expect("Stale lock acquired cleanly");
    drop(lock);
}
