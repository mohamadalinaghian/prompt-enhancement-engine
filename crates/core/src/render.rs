use crate::error::CoreError;
use crate::templates::contract::TemplateContract;
use std::collections::HashMap;

pub struct XmlRenderer;

impl XmlRenderer {
    /// Escape XML special characters in dynamic content to prevent prompt injection from breaking tag balance.
    fn escape_xml_text(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    /// Render the final XML prompt according to the template contract and provided dynamic fields.
    pub fn render(
        contract: &TemplateContract,
        fields: &HashMap<String, String>,
    ) -> Result<String, CoreError> {
        // 1. Verify required fields are present and non-empty
        for field in contract.required_fields() {
            match fields.get(field.name) {
                Some(val) if !val.trim().is_empty() => {}
                _ => {
                    return Err(CoreError::TemplateContractMismatch(format!(
                        "Template '{}' is missing required field '{}'",
                        contract.template_id.as_str(),
                        field.name
                    )));
                }
            }
        }

        let mut out = String::with_capacity(2048);

        // 2. Render <task session="...">
        if let Some(task) = fields.get("task") {
            out.push_str(&format!(
                "<task session=\"{}\">\n",
                contract.template_id.as_str()
            ));
            out.push_str(Self::escape_xml_text(task.trim()).as_str());
            out.push_str("\n</task>\n\n");
        }

        // 3. Render <acceptance_criteria>
        if let Some(ac) = fields.get("acceptance_criteria") {
            out.push_str("<acceptance_criteria>\n");
            out.push_str(Self::escape_xml_text(ac.trim()).as_str());
            out.push_str("\n</acceptance_criteria>\n\n");
        }

        // 4. Render <context> if provided or required
        if let Some(ctx) = fields.get("context") {
            if !ctx.trim().is_empty() {
                out.push_str("<context>\n");
                out.push_str(Self::escape_xml_text(ctx.trim()).as_str());
                out.push_str("\n</context>\n\n");
            }
        }

        // 5. Render <scope>
        out.push_str("<scope>\n");
        if let Some(rf) = fields.get("relevant_files") {
            if !rf.trim().is_empty() {
                out.push_str("<relevant_files>");
                out.push_str(Self::escape_xml_text(rf.trim()).as_str());
                out.push_str("</relevant_files>\n");
            }
        }
        out.push_str("<out_of_scope>");
        out.push_str(contract.static_sections.out_of_scope.trim());
        out.push_str("</out_of_scope>\n");
        out.push_str("</scope>\n\n");

        // 6. Render <rules>
        out.push_str("<rules>\n");
        out.push_str(contract.static_sections.rules.trim());
        out.push_str("\n</rules>\n\n");

        // 7. Render <verify> if present in contract
        if let Some(verify_template) = contract.static_sections.verify {
            out.push_str("<verify>\n");
            let mut verify_body = verify_template.trim().to_string();

            // Replace dynamic command placeholders if provided in fields
            for (key, val) in fields {
                let placeholder = format!("{{{{{}}}}}", key);
                verify_body = verify_body.replace(&placeholder, val.trim());
            }

            out.push_str(&verify_body);
            out.push_str("\n</verify>\n\n");
        }

        // 8. Render <report>
        out.push_str("<report>\n");
        out.push_str(contract.static_sections.report.trim());
        out.push_str("\n</report>\n");

        // 9. Final structural validation: verify well-formed tag balance
        Self::validate_xml_balance(&out)?;

        Ok(out)
    }

    /// Validates that required top-level XML tags are properly balanced.
    fn validate_xml_balance(xml: &str) -> Result<(), CoreError> {
        let tags = ["task", "acceptance_criteria", "scope", "rules", "report"];

        for tag in tags {
            let close = format!("</{}>", tag);
            let open_exact = format!("<{}>", tag);
            let open_attr = format!("<{} ", tag);

            let open_count = xml.matches(&open_exact).count() + xml.matches(&open_attr).count();
            let close_count = xml.matches(&close).count();

            if open_count != close_count {
                return Err(CoreError::ValidationFailed(format!(
                    "Unbalanced XML tag '{}': found {} opening and {} closing",
                    tag, open_count, close_count
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::definitions::*;

    #[test]
    fn test_render_implementation_refactoring() {
        let contract = &IMPL_REFACTOR_CONTRACT;
        let mut fields = HashMap::new();
        fields.insert(
            "task".to_string(),
            "Add caching layer to repository".to_string(),
        );
        fields.insert(
            "acceptance_criteria".to_string(),
            "- Cache hits under 5ms\n- Memory bounded to 64MB".to_string(),
        );
        fields.insert(
            "relevant_files".to_string(),
            "crates/core/src/cache.rs".to_string(),
        );
        fields.insert(
            "context".to_string(),
            "- Using lru crate\n- Thread-safe Arc<Mutex<...>>".to_string(),
        );

        let rendered = XmlRenderer::render(contract, &fields).unwrap();

        assert!(rendered.starts_with("<task session=\"implementation_refactoring\">\nAdd caching layer to repository\n</task>"));
        assert!(rendered.contains("<acceptance_criteria>\n- Cache hits under 5ms\n- Memory bounded to 64MB\n</acceptance_criteria>"));
        assert!(rendered.contains(
            "<context>\n- Using lru crate\n- Thread-safe Arc&lt;Mutex&lt;...&gt;&gt;\n</context>"
        ));
        assert!(rendered.contains("<relevant_files>crates/core/src/cache.rs</relevant_files>"));
        assert!(rendered.contains("<rules>"));
        assert!(rendered.contains("<verify>"));
        assert!(rendered.contains("<report>"));
    }

    #[test]
    fn test_render_missing_required_field_fails() {
        let contract = &IMPL_REFACTOR_CONTRACT;
        let mut fields = HashMap::new();
        fields.insert("task".to_string(), "Implement something".to_string());
        // Missing acceptance_criteria and relevant_files

        let err = XmlRenderer::render(contract, &fields).unwrap_err();
        match err {
            CoreError::TemplateContractMismatch(msg) => {
                assert!(msg.contains("acceptance_criteria"));
            }
            other => panic!("Expected TemplateContractMismatch, got {:?}", other),
        }
    }

    #[test]
    fn test_render_read_only_question_omits_verify() {
        let contract = &QUESTION_CONTRACT;
        let mut fields = HashMap::new();
        fields.insert(
            "task".to_string(),
            "How does single-flight locking work?".to_string(),
        );
        fields.insert(
            "acceptance_criteria".to_string(),
            "Direct explanation with file citations".to_string(),
        );

        let rendered = XmlRenderer::render(contract, &fields).unwrap();
        assert!(!rendered.contains("<verify>"));
        assert!(rendered.contains("<rules>"));
        assert!(rendered.contains("<out_of_scope>Making code edits, modifying repository files, destructive commands</out_of_scope>"));
    }

    #[test]
    fn test_render_debug_bugfix_replaces_verify_placeholder() {
        let contract = &DEBUG_BUGFIX_CONTRACT;
        let mut fields = HashMap::new();
        fields.insert(
            "task".to_string(),
            "Fix off-by-one error in parser".to_string(),
        );
        fields.insert(
            "acceptance_criteria".to_string(),
            "Regression test passes".to_string(),
        );
        fields.insert("context".to_string(), "Panic at line 42".to_string());
        fields.insert(
            "relevant_files".to_string(),
            "crates/core/src/parser.rs".to_string(),
        );
        fields.insert(
            "target_test_command".to_string(),
            "cargo test -p pe-core test_parse_empty".to_string(),
        );

        let rendered = XmlRenderer::render(contract, &fields).unwrap();
        assert!(rendered.contains("cargo test -p pe-core test_parse_empty"));
        assert!(!rendered.contains("{{target_test_command}}"));
    }
}
