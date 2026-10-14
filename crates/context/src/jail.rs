use pe_core::CoreError;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Safe jailed filesystem access restricted to a verified project root (D-018).
#[derive(Debug, Clone)]
pub struct ProjectJail {
    canonical_root: PathBuf,
}

impl ProjectJail {
    /// Create and initialize a new ProjectJail after validating and canonicalizing the root.
    pub fn new(root: impl AsRef<Path>) -> Result<Self, CoreError> {
        let path = root.as_ref();
        let canonical_root = path.canonicalize().map_err(|e| {
            CoreError::ProjectUnavailable(format!(
                "Failed to canonicalize project root '{}': {}",
                path.display(),
                e
            ))
        })?;

        if !canonical_root.is_dir() {
            return Err(CoreError::ProjectUnavailable(format!(
                "Project root '{}' is not a directory",
                canonical_root.display()
            )));
        }

        Ok(Self { canonical_root })
    }

    /// Access the verified canonical root path.
    pub fn root(&self) -> &Path {
        &self.canonical_root
    }

    /// Resolve and verify that a given path resides strictly within the canonical root.
    /// Re-canonicalizes the path to detect symlinks escaping the jail.
    pub fn resolve_and_verify(&self, target: impl AsRef<Path>) -> Result<PathBuf, CoreError> {
        let target_path = target.as_ref();

        // If target is relative, join to root; if absolute, attempt to canonicalize directly
        let full_path = if target_path.is_relative() {
            self.canonical_root.join(target_path)
        } else {
            target_path.to_path_buf()
        };

        let canonical_target = full_path.canonicalize().map_err(|e| {
            CoreError::ProjectUnavailable(format!(
                "Failed to resolve path '{}': {}",
                full_path.display(),
                e
            ))
        })?;

        if !canonical_target.starts_with(&self.canonical_root) {
            return Err(CoreError::SafetyBlocked(format!(
                "Path '{}' escapes canonical project root '{}'",
                canonical_target.display(),
                self.canonical_root.display()
            )));
        }

        Ok(canonical_target)
    }

    /// Get relative path from canonical root.
    pub fn relative_path(&self, target: &Path) -> PathBuf {
        target
            .strip_prefix(&self.canonical_root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| target.to_path_buf())
    }

    /// Read file content safely, verifying jail containment, size limits, and returning SHA-256 hash.
    pub fn read_file(
        &self,
        target: impl AsRef<Path>,
        max_bytes: usize,
    ) -> Result<(String, String), CoreError> {
        let verified_path = self.resolve_and_verify(target)?;

        let metadata = fs::metadata(&verified_path).map_err(|e| {
            CoreError::ProjectUnavailable(format!(
                "Failed to read metadata for '{}': {}",
                verified_path.display(),
                e
            ))
        })?;

        if !metadata.is_file() {
            return Err(CoreError::SafetyBlocked(format!(
                "Target '{}' is not a regular file",
                verified_path.display()
            )));
        }

        let file_size = metadata.len() as usize;
        if file_size > max_bytes {
            return Err(CoreError::SafetyBlocked(format!(
                "File '{}' exceeds byte limit ({} > {} bytes)",
                verified_path.display(),
                file_size,
                max_bytes
            )));
        }

        let bytes = fs::read(&verified_path).map_err(|e| {
            CoreError::ProjectUnavailable(format!(
                "Failed to read file '{}': {}",
                verified_path.display(),
                e
            ))
        })?;

        // Calculate SHA-256 hash for provenance
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let sha256_hash = format!("{:x}", hasher.finalize());

        let content = String::from_utf8_lossy(&bytes).to_string();

        Ok((content, sha256_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_jail_containment_valid_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        {
            let mut f = File::create(&file_path).unwrap();
            f.write_all(b"fn main() {}").unwrap();
        }

        let jail = ProjectJail::new(dir.path()).unwrap();
        let (content, hash) = jail.read_file("main.rs", 1024).unwrap();
        assert_eq!(content, "fn main() {}");
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_jail_blocks_parent_directory_traversal() {
        let dir = tempdir().unwrap();
        let outside_dir = tempdir().unwrap();
        let outside_file = outside_dir.path().join("secret.txt");
        {
            let mut f = File::create(&outside_file).unwrap();
            f.write_all(b"supersecret").unwrap();
        }

        let jail = ProjectJail::new(dir.path()).unwrap();
        let err = jail.read_file(&outside_file, 1024).unwrap_err();

        match err {
            CoreError::SafetyBlocked(msg) => {
                assert!(msg.contains("escapes canonical project root"));
            }
            other => panic!("Expected SafetyBlocked, got {:?}", other),
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_jail_blocks_symlink_escape() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().unwrap();
        let outside_dir = tempdir().unwrap();
        let outside_file = outside_dir.path().join("outside.txt");
        {
            let mut f = File::create(&outside_file).unwrap();
            f.write_all(b"outside").unwrap();
        }

        let escaping_symlink = dir.path().join("symlink_to_outside.txt");
        symlink(&outside_file, &escaping_symlink).unwrap();

        let jail = ProjectJail::new(dir.path()).unwrap();
        let err = jail.read_file("symlink_to_outside.txt", 1024).unwrap_err();

        match err {
            CoreError::SafetyBlocked(msg) => {
                assert!(msg.contains("escapes canonical project root"));
            }
            other => panic!("Expected SafetyBlocked, got {:?}", other),
        }
    }
}
