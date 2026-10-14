use pe_core::parser::{CLARIFY_BEGIN_SENTINEL, CLARIFY_END_SENTINEL, CLARIFY_HEADER};

/// Builds a structured clarification block prepended to the original input (workflow-integration.md §8).
pub fn format_clarification_block(
    question: &str,
    blocking_reason: &str,
    original_input: &str,
) -> String {
    format!(
        "{}\nDelete this whole block, add your answer to your prompt, and run the shortcut again.\n{}\nQuestion: {}\nWhy it blocks: {}\n{}\n{}",
        CLARIFY_HEADER,
        CLARIFY_BEGIN_SENTINEL,
        question.trim(),
        blocking_reason.trim(),
        CLARIFY_END_SENTINEL,
        original_input
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pe_core::InputParser;

    #[test]
    fn test_format_clarification_block_detectable_by_parser() {
        let original = "~/my_proj\nFix bug in parser";
        let formatted = format_clarification_block(
            "Which parser function is failing?",
            "Multiple parser implementations exist in the repo.",
            original,
        );

        assert!(formatted.starts_with(CLARIFY_HEADER));
        assert!(formatted.contains(CLARIFY_BEGIN_SENTINEL));
        assert!(formatted.contains("Question: Which parser function is failing?"));
        assert!(formatted.contains(CLARIFY_END_SENTINEL));
        assert!(formatted.ends_with(original));

        // When re-fed to InputParser, must trigger StaleClarificationMarker error
        let parse_res = InputParser::parse(&formatted, 4096);
        assert_eq!(
            parse_res.unwrap_err(),
            pe_core::CoreError::StaleClarificationMarker
        );
    }
}
