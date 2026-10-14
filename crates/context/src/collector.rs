use crate::budget::BudgetTracker;
use crate::evidence::{Classification, EvidenceBundle, EvidenceItem, Provenance};
use crate::jail::ProjectJail;
use crate::scanner::FileScanner;
use pe_core::config::Config;
use pe_core::secrets::SecretScanner;
use pe_core::templates::TemplateContract;
use pe_core::types::ValidatedProjectRoot;
use pe_core::{CoreError, EvidenceStatus};
use regex::Regex;

/// Field-driven local repository context collector (context-collection.md §3).
pub struct FieldDrivenCollector;

impl FieldDrivenCollector {
    /// Collect minimal verified evidence driven by template needs and budget bounds.
    pub fn collect(
        contract: &TemplateContract,
        task_text: &str,
        project_root: Option<&ValidatedProjectRoot>,
        config: &Config,
    ) -> Result<EvidenceBundle, CoreError> {
        // 1. Without a valid project root or if template forbids evidence, skip collection
        let root = match project_root {
            Some(r) => r,
            None => {
                return Ok(EvidenceBundle::empty(
                    contract.template_id,
                    EvidenceStatus::NoCollection,
                ));
            }
        };

        if !contract.has_evidence_allowed_field() {
            return Ok(EvidenceBundle::empty(
                contract.template_id,
                EvidenceStatus::NoCollection,
            ));
        }

        // 2. Initialize safe jail and budget tracker
        let jail = ProjectJail::new(root.path())?;
        let mut budget = BudgetTracker::new(config.budgets.clone());
        let mut scanner = FileScanner::new();

        // 3. Extract path hints directly named in the user's task text
        let hints = Self::extract_path_hints(task_text);

        let mut evidence_items = Vec::new();
        let mut item_counter = 1;

        // 4. Bounded collection loop
        while budget.can_iterate() && !budget.is_exhausted() {
            budget.record_iteration();

            let candidates = scanner.discover_candidates(
                &jail,
                &hints,
                &config.exclusions,
                config.budgets.max_inspected_files,
            )?;

            if candidates.is_empty() {
                break;
            }

            for candidate in candidates {
                if !budget.can_inspect_file() {
                    break;
                }

                scanner.mark_inspected(candidate.clone());
                budget.record_file_inspected();

                // Read file within byte budget
                let max_read = config
                    .budgets
                    .max_read_bytes
                    .saturating_sub(budget.total_bytes_read);
                if max_read == 0 {
                    break;
                }

                let (raw_content, hash) = match jail.read_file(&candidate, max_read) {
                    Ok(res) => res,
                    Err(_) => continue,
                };

                // Irreversibly redact any repository secrets (D-016)
                let (redacted_content, _) = SecretScanner::redact(&raw_content);

                budget.record_read(raw_content.len(), &redacted_content);

                let rel_path = jail.relative_path(&candidate);
                evidence_items.push(EvidenceItem {
                    id: format!("ev_{}", item_counter),
                    content: redacted_content,
                    classification: Classification::DirectObservation,
                    provenance: Provenance {
                        relative_path: rel_path,
                        line_range: None,
                        sha256_hash: hash,
                        modified_at: None,
                    },
                    relevance_rationale: "Discovered file matching template scope".to_string(),
                    need: "repository_context".to_string(),
                });

                item_counter += 1;
            }
        }

        let status = if budget.is_exhausted() && evidence_items.is_empty() {
            EvidenceStatus::BudgetExhausted
        } else {
            EvidenceStatus::SufficientContext
        };

        // Determine collection profile (first allowed profile or targeted_file)
        let profile = contract
            .allowed_profiles
            .first()
            .copied()
            .unwrap_or(pe_core::templates::CollectionProfile::TargetedFile);

        Ok(EvidenceBundle {
            template_id: contract.template_id,
            profile,
            items: evidence_items,
            status,
        })
    }

    /// Extract file path hints mentioned in the prompt text.
    fn extract_path_hints(text: &str) -> Vec<String> {
        let re = Regex::new(r"\b([a-zA-Z0-9_\-/\\]+\.[a-zA-Z0-9]+)\b").unwrap();
        let mut hints = Vec::new();

        for cap in re.captures_iter(text) {
            let matched = cap[1].to_string();
            // Filter out obvious noise like decimal numbers or URLs
            if !matched.starts_with("http") && !hints.contains(&matched) {
                hints.push(matched);
            }
        }

        hints
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pe_core::templates::definitions::{GIT_CONTRACT, IMPL_REFACTOR_CONTRACT};
    use std::fs::write;
    use tempfile::tempdir;

    #[test]
    fn test_collect_no_collection_when_root_none() {
        let contract = &IMPL_REFACTOR_CONTRACT;
        let bundle =
            FieldDrivenCollector::collect(contract, "Add caching", None, &Config::default())
                .unwrap();

        assert_eq!(bundle.status, EvidenceStatus::NoCollection);
        assert!(bundle.is_empty());
    }

    #[test]
    fn test_collect_no_collection_when_template_has_no_evidence_fields() {
        let dir = tempdir().unwrap();
        let root = ValidatedProjectRoot::new(dir.path().to_path_buf());

        // GIT_CONTRACT has only UserTextOnly fields
        let contract = &GIT_CONTRACT;
        let bundle = FieldDrivenCollector::collect(
            contract,
            "commit changes",
            Some(&root),
            &Config::default(),
        )
        .unwrap();

        assert_eq!(bundle.status, EvidenceStatus::NoCollection);
        assert!(bundle.is_empty());
    }

    #[test]
    fn test_collect_reads_targeted_files_and_redacts_secrets() {
        let dir = tempdir().unwrap();
        let root_path = dir.path().canonicalize().unwrap();
        let root = ValidatedProjectRoot::new(root_path.clone());

        // Create a file mentioned in task text
        let cache_file = root_path.join("cache.rs");
        write(
            &cache_file,
            b"// cache module\nlet key = \"sk-1234567890abcdef1234567890\";",
        )
        .unwrap();

        let contract = &IMPL_REFACTOR_CONTRACT;
        let task = "Refactor cache.rs to improve speed";

        let bundle =
            FieldDrivenCollector::collect(contract, task, Some(&root), &Config::default()).unwrap();

        assert_eq!(bundle.status, EvidenceStatus::SufficientContext);
        assert_eq!(bundle.len(), 1);

        let item = &bundle.items[0];
        assert_eq!(item.provenance.relative_path.to_str().unwrap(), "cache.rs");
        assert!(!item.content.contains("sk-1234567890abcdef1234567890"));
        assert!(item.content.contains("[[SECRET_PE_"));
        assert_eq!(item.classification, Classification::DirectObservation);
    }
}
