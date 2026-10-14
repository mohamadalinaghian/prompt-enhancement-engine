use crate::jail::ProjectJail;
use pe_core::config::ExclusionConfig;
use pe_core::CoreError;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Safe in-process candidate file discovery engine (context-collection.md §4, §8).
#[derive(Debug, Default)]
pub struct FileScanner {
    /// Inspected set to ensure no duplicate file reads across collection iterations.
    inspected_set: HashSet<PathBuf>,
}

impl FileScanner {
    pub fn new() -> Self {
        Self {
            inspected_set: HashSet::new(),
        }
    }

    /// Mark a path as inspected.
    pub fn mark_inspected(&mut self, path: PathBuf) {
        self.inspected_set.insert(path);
    }

    /// Check if a path has already been inspected.
    pub fn is_inspected(&self, path: &Path) -> bool {
        self.inspected_set.contains(path)
    }

    /// Discover eligible candidate files under the jail, filtered by exclusions and user hints.
    pub fn discover_candidates(
        &mut self,
        jail: &ProjectJail,
        hints: &[String],
        exclusions: &ExclusionConfig,
        max_candidates: usize,
    ) -> Result<Vec<PathBuf>, CoreError> {
        let mut candidates = Vec::new();
        let root = jail.root();

        // 1. First, check any explicit file paths named by the user
        for hint in hints {
            let prospective = root.join(hint);
            if prospective.is_file() {
                if let Ok(verified) = jail.resolve_and_verify(&prospective) {
                    if !self.is_inspected(&verified)
                        && !self.is_excluded(&verified, exclusions)
                        && !self.is_binary(&verified)
                    {
                        candidates.push(verified);
                        if candidates.len() >= max_candidates {
                            return Ok(candidates);
                        }
                    }
                }
            }
        }

        // 2. Discover files via bounded in-process traversal using WalkDir
        let walker = WalkDir::new(root)
            .max_depth(5)
            .follow_links(false)
            .into_iter();

        for entry in walker.filter_entry(|e| !self.is_entry_excluded(e.path(), exclusions)) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if path.is_file() {
                if let Ok(verified) = jail.resolve_and_verify(path) {
                    if !self.is_inspected(&verified)
                        && !self.is_excluded(&verified, exclusions)
                        && !self.is_binary(&verified)
                        && !candidates.contains(&verified)
                    {
                        candidates.push(verified);
                        if candidates.len() >= max_candidates {
                            break;
                        }
                    }
                }
            }
        }

        Ok(candidates)
    }

    fn is_entry_excluded(&self, path: &Path, exclusions: &ExclusionConfig) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            for ignored_dir in &exclusions.ignored_directories {
                if name == ignored_dir {
                    return true;
                }
            }
        }
        false
    }

    fn is_excluded(&self, path: &Path, exclusions: &ExclusionConfig) -> bool {
        let path_str = path.to_string_lossy();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        // Check directory exclusions
        for dir in &exclusions.ignored_directories {
            if path_str.contains(&format!("/{}/", dir)) || path_str.ends_with(&format!("/{}", dir))
            {
                return true;
            }
        }

        // Check file pattern exclusions
        for pat in &exclusions.ignored_file_patterns {
            if let Some(suffix) = pat.strip_prefix('*') {
                if file_name.ends_with(suffix) {
                    return true;
                }
            } else if let Some(prefix) = pat.strip_suffix('*') {
                if file_name.starts_with(prefix) {
                    return true;
                }
            } else if file_name == pat {
                return true;
            }
        }

        false
    }

    /// Check if a file appears to be binary by checking for null bytes in initial 512 bytes.
    fn is_binary(&self, path: &Path) -> bool {
        if let Ok(mut f) = File::open(path) {
            let mut buf = [0u8; 512];
            if let Ok(n) = f.read(&mut buf) {
                return buf[..n].contains(&0);
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir, write};
    use tempfile::tempdir;

    #[test]
    fn test_scanner_finds_files_and_respects_exclusions() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        write(root.join("Cargo.toml"), b"[package]\nname = \"foo\"").unwrap();
        write(root.join(".env"), b"SECRET=123").unwrap();

        let target_dir = root.join("target");
        create_dir(&target_dir).unwrap();
        write(target_dir.join("build.rs"), b"// build").unwrap();

        let jail = ProjectJail::new(root).unwrap();
        let exclusions = ExclusionConfig::default();
        let mut scanner = FileScanner::new();

        let candidates = scanner
            .discover_candidates(&jail, &[], &exclusions, 10)
            .unwrap();

        let candidate_names: Vec<String> = candidates
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();

        assert!(candidate_names.contains(&"Cargo.toml".to_string()));
        assert!(!candidate_names.contains(&".env".to_string()));
        assert!(!candidate_names.contains(&"build.rs".to_string()));
    }

    #[test]
    fn test_scanner_inspected_set_deduplication() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let file = root.join("lib.rs");
        write(&file, b"pub fn hello() {}").unwrap();

        let jail = ProjectJail::new(root).unwrap();
        let exclusions = ExclusionConfig::default();
        let mut scanner = FileScanner::new();

        let c1 = scanner
            .discover_candidates(&jail, &[], &exclusions, 10)
            .unwrap();
        assert_eq!(c1.len(), 1);

        scanner.mark_inspected(c1[0].clone());

        let c2 = scanner
            .discover_candidates(&jail, &[], &exclusions, 10)
            .unwrap();
        assert!(c2.is_empty());
    }
}
