use crate::error::CoreError;
use crate::types::{OpaqueBlock, ParsedInput, ValidatedProjectRoot};
use regex::Regex;
use std::path::PathBuf;

pub const CLARIFY_BEGIN_SENTINEL: &str = "[[PE-CLARIFY-v1:BEGIN]]";
pub const CLARIFY_END_SENTINEL: &str = "[[PE-CLARIFY-v1:END]]";
pub const CLARIFY_HEADER: &str = "⚠ PROMPT ENHANCER — CLARIFICATION NEEDED";

pub struct InputParser;

impl InputParser {
    pub fn parse(input: &str, max_bytes: usize) -> Result<ParsedInput, CoreError> {
        let byte_len = input.len();
        if byte_len > max_bytes {
            return Err(CoreError::InputTooLarge {
                actual: byte_len,
                max: max_bytes,
            });
        }

        if input.trim().is_empty() {
            return Err(CoreError::InputEmpty);
        }

        // Check for stale clarification marker anchored at the start
        let trimmed_start = input.trim_start();
        if trimmed_start.starts_with(CLARIFY_HEADER)
            || trimmed_start.starts_with(CLARIFY_BEGIN_SENTINEL)
        {
            return Err(CoreError::StaleClarificationMarker);
        }

        // Split lines to inspect first line
        let mut lines = input.lines();
        let first_line = lines.next().unwrap_or("").trim();

        let (project_root, remaining_text) = if first_line.starts_with("~/") {
            let expanded_path = Self::expand_tilde(first_line)?;
            let canonical = expanded_path.canonicalize().map_err(|e| {
                CoreError::ProjectUnavailable(format!(
                    "Path '{}' could not be canonicalized: {}",
                    first_line, e
                ))
            })?;

            if !canonical.is_dir() {
                return Err(CoreError::ProjectUnavailable(format!(
                    "Path '{}' is not a directory",
                    first_line
                )));
            }

            // Consume first line and join the rest
            let rest: Vec<&str> = lines.collect();
            let remainder = rest.join("\n");
            (Some(ValidatedProjectRoot::new(canonical)), remainder)
        } else {
            (None, input.to_string())
        };

        // Extract opaque fenced blocks
        let (task_text, opaque_blocks) = Self::extract_opaque_blocks(&remaining_text);

        Ok(ParsedInput {
            raw_input: input.to_string(),
            task_text,
            project_root,
            opaque_blocks,
        })
    }

    fn expand_tilde(path_str: &str) -> Result<PathBuf, CoreError> {
        if let Some(stripped) = path_str.strip_prefix("~/") {
            let home = dirs::home_dir().ok_or_else(|| {
                CoreError::ProjectUnavailable("Unable to resolve user home directory".to_string())
            })?;
            Ok(home.join(stripped))
        } else {
            Ok(PathBuf::from(path_str))
        }
    }

    fn extract_opaque_blocks(text: &str) -> (String, Vec<OpaqueBlock>) {
        // Match ```...``` or ~~~...~~~ including language tags and multiline content
        let re = Regex::new(r"(?s)(```[^\n]*\n.*?```|~~~[^\n]*\n.*?~~~)").unwrap();

        let mut opaque_blocks = Vec::new();
        let mut block_index = 0;

        let task_text = re
            .replace_all(text, |caps: &regex::Captures| {
                let id = format!("[[PE_OPAQUE_BLOCK_{}]]", block_index);
                let matched = caps[0].to_string();
                opaque_blocks.push(OpaqueBlock {
                    id: id.clone(),
                    original_text: matched,
                });
                block_index += 1;
                id
            })
            .into_owned();

        (task_text, opaque_blocks)
    }

    pub fn restore_opaque_blocks(text: &str, blocks: &[OpaqueBlock]) -> String {
        let mut restored = text.to_string();
        for block in blocks {
            restored = restored.replace(&block.id, &block.original_text);
        }
        restored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_empty() {
        assert_eq!(InputParser::parse("", 1024), Err(CoreError::InputEmpty));
        assert_eq!(
            InputParser::parse("   \n\t  ", 1024),
            Err(CoreError::InputEmpty)
        );
    }

    #[test]
    fn test_stale_clarification_sentinel() {
        let input = format!(
            "{}\nQuestion: foo\n{}",
            CLARIFY_HEADER, CLARIFY_BEGIN_SENTINEL
        );
        assert_eq!(
            InputParser::parse(&input, 1024),
            Err(CoreError::StaleClarificationMarker)
        );

        let input_begin = format!("{}\nQuestion: foo", CLARIFY_BEGIN_SENTINEL);
        assert_eq!(
            InputParser::parse(&input_begin, 1024),
            Err(CoreError::StaleClarificationMarker)
        );
    }

    #[test]
    fn test_opaque_block_extraction_and_restoration() {
        let text = "Here is some code:\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\nAnd more text.";
        let (task, blocks) = InputParser::extract_opaque_blocks(text);

        assert!(task.contains("[[PE_OPAQUE_BLOCK_0]]"));
        assert!(!task.contains("println!(\"hello\")"));
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id, "[[PE_OPAQUE_BLOCK_0]]");

        let restored = InputParser::restore_opaque_blocks(&task, &blocks);
        assert_eq!(restored, text);
    }

    #[test]
    fn test_project_path_extraction() {
        let temp = tempdir().unwrap();
        let canonical_dir = temp.path().canonicalize().unwrap();

        // Simulate a home dir relative path
        if let Some(home) = dirs::home_dir() {
            if let Ok(rel) = canonical_dir.strip_prefix(&home) {
                let input = format!("~/{}\nFix the authentication bug", rel.display());
                let parsed = InputParser::parse(&input, 1024).unwrap();
                assert_eq!(parsed.project_root.unwrap().path(), canonical_dir);
                assert_eq!(parsed.task_text, "Fix the authentication bug");
            }
        }
    }
}
