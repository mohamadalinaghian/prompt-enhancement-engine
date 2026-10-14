use pe_core::TerminalStatus;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Single-flight lock preventing overlapping runs (workflow-integration.md §5, D-014).
#[derive(Debug)]
pub struct SingleFlightLock {
    lock_path: PathBuf,
}

impl SingleFlightLock {
    /// Acquire an exclusive single-flight lock or return TerminalStatus::Busy.
    pub fn acquire(custom_path: Option<&Path>) -> Result<Self, TerminalStatus> {
        let lock_path = match custom_path {
            Some(p) => p.to_path_buf(),
            None => Self::default_lock_path(),
        };

        if let Some(parent) = lock_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let current_pid = std::process::id() as i32;

        if lock_path.exists() {
            if let Ok(content) = fs::read_to_string(&lock_path) {
                if let Ok(existing_pid) = content.trim().parse::<i32>() {
                    if Self::is_process_alive(existing_pid) {
                        return Err(TerminalStatus::Busy);
                    }
                }
            }
            // Stale lock from terminated process: remove and re-acquire
            let _ = fs::remove_file(&lock_path);
        }

        let mut file = File::create(&lock_path).map_err(|_| TerminalStatus::Busy)?;
        writeln!(file, "{}", current_pid).map_err(|_| TerminalStatus::Busy)?;

        Ok(Self { lock_path })
    }

    pub fn default_lock_path() -> PathBuf {
        if let Some(mut state_dir) = dirs::state_dir() {
            state_dir.push("prompt_enhancer/pe.lock");
            state_dir
        } else {
            let mut tmp = std::env::temp_dir();
            tmp.push(format!("pe_{}.lock", std::process::id()));
            tmp
        }
    }

    /// Check if a process is still alive on macOS/Unix using kill(pid, 0).
    fn is_process_alive(pid: i32) -> bool {
        #[cfg(unix)]
        {
            unsafe {
                if libc::kill(pid, 0) == 0 {
                    return true;
                }
                let err = *libc::__error();
                // EPERM means the process exists but is owned by another user
                err == libc::EPERM
            }
        }
        #[cfg(not(unix))]
        {
            true
        }
    }
}

impl Drop for SingleFlightLock {
    fn drop(&mut self) {
        if self.lock_path.exists() {
            let current_pid = std::process::id() as i32;
            if let Ok(content) = fs::read_to_string(&self.lock_path) {
                if let Ok(existing_pid) = content.trim().parse::<i32>() {
                    if existing_pid == current_pid {
                        let _ = fs::remove_file(&self.lock_path);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_single_flight_lock_lifecycle() {
        let dir = tempdir().unwrap();
        let lock_file = dir.path().join("pe.lock");

        {
            let lock1 = SingleFlightLock::acquire(Some(&lock_file)).unwrap();
            assert!(lock_file.exists());

            // Second lock while first is held must return Busy
            let lock2 = SingleFlightLock::acquire(Some(&lock_file));
            assert_eq!(lock2.unwrap_err(), TerminalStatus::Busy);

            drop(lock1);
        }

        // After drop, lock file is released and can be acquired again
        let lock3 = SingleFlightLock::acquire(Some(&lock_file));
        assert!(lock3.is_ok());
    }

    #[test]
    fn test_stale_lock_cleanup() {
        let dir = tempdir().unwrap();
        let lock_file = dir.path().join("pe.lock");

        // Write a dead PID (e.g. 999999999)
        fs::write(&lock_file, "999999999\n").unwrap();

        // Must clean up stale lock and succeed
        let lock = SingleFlightLock::acquire(Some(&lock_file));
        assert!(lock.is_ok());
    }
}
