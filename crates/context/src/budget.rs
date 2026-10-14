use pe_core::config::BudgetConfig;
use serde::{Deserialize, Serialize};

/// Monitors and enforces resource budgets during context collection (context-collection.md §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetTracker {
    pub files_inspected: usize,
    pub total_bytes_read: usize,
    pub estimated_tokens: usize,
    pub iterations: usize,
    pub config: BudgetConfig,
}

impl BudgetTracker {
    pub fn new(config: BudgetConfig) -> Self {
        Self {
            files_inspected: 0,
            total_bytes_read: 0,
            estimated_tokens: 0,
            iterations: 0,
            config,
        }
    }

    /// Check if another file can be inspected without exceeding budget.
    pub fn can_inspect_file(&self) -> bool {
        self.files_inspected < self.config.max_inspected_files
    }

    /// Check if reading `additional_bytes` would exceed the byte budget.
    pub fn can_read_bytes(&self, additional_bytes: usize) -> bool {
        self.total_bytes_read.saturating_add(additional_bytes) <= self.config.max_read_bytes
    }

    /// Check if another collection loop iteration can be performed.
    pub fn can_iterate(&self) -> bool {
        self.iterations < self.config.max_collection_iterations
    }

    /// Record an inspected candidate file.
    pub fn record_file_inspected(&mut self) {
        self.files_inspected += 1;
    }

    /// Record bytes and tokens consumed by reading a file or excerpt.
    pub fn record_read(&mut self, bytes: usize, text: &str) {
        self.total_bytes_read = self.total_bytes_read.saturating_add(bytes);
        let tokens = text.chars().count().div_ceil(4);
        self.estimated_tokens = self.estimated_tokens.saturating_add(tokens);
    }

    /// Record a collection loop iteration.
    pub fn record_iteration(&mut self) {
        self.iterations += 1;
    }

    /// Check if any hard budget threshold has been reached.
    pub fn is_exhausted(&self) -> bool {
        self.files_inspected >= self.config.max_inspected_files
            || self.total_bytes_read >= self.config.max_read_bytes
            || self.estimated_tokens >= self.config.max_payload_tokens
            || self.iterations >= self.config.max_collection_iterations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_tracker_limits() {
        let cfg = BudgetConfig {
            max_inspected_files: 2,
            max_read_bytes: 100,
            max_collection_iterations: 2,
            ..Default::default()
        };

        let mut tracker = BudgetTracker::new(cfg);
        assert!(tracker.can_inspect_file());
        assert!(tracker.can_iterate());
        assert!(!tracker.is_exhausted());

        tracker.record_file_inspected();
        tracker.record_iteration();
        assert!(!tracker.is_exhausted());

        tracker.record_file_inspected();
        assert!(!tracker.can_inspect_file());
        assert!(tracker.is_exhausted());
    }

    #[test]
    fn test_budget_byte_limit() {
        let cfg = BudgetConfig {
            max_read_bytes: 50,
            ..Default::default()
        };

        let mut tracker = BudgetTracker::new(cfg);
        assert!(tracker.can_read_bytes(30));
        assert!(!tracker.can_read_bytes(60));

        tracker.record_read(40, "sample text of 40 bytes");
        assert!(!tracker.can_read_bytes(20));
        assert!(tracker.can_read_bytes(10));
    }
}
