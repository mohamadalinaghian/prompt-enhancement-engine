use crate::templates::contract::{
    CollectionProfile, EvidencePolicy, FieldContract, StaticSections, TemplateContract,
};
use crate::types::{SessionKind, TemplateId};

// --- IMPLEMENTATION & REFACTORING ---
static IMPL_REFACTOR_PROFILES: [CollectionProfile; 2] =
    [CollectionProfile::TargetedFile, CollectionProfile::None];
static IMPL_REFACTOR_FIELDS: [FieldContract; 4] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description:
            "Specific feature or refactoring objective with exact naming, routes, and behavior",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Observable, testable criteria for completion including edge cases",
    },
    FieldContract {
        name: "context",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Relevant interfaces, architectural guidelines, caller code, or dependencies",
    },
    FieldContract {
        name: "relevant_files",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Specific file paths in scope for editing or reference",
    },
];
static IMPL_REFACTOR_STATIC: StaticSections = StaticSections {
    rules: "- Be EXTREMELY concise. The user skims text. Do not explain how code works unless asked.\n- Follow existing patterns in the repo; check neighboring files first.\n- No mock data or swallowed errors; handle failures explicitly.\n- Ask before: destructive actions, schema or public API changes.\n- Preserve existing comments and docstrings unrelated to your changes.\n- Never delete or weaken existing tests to make a build pass.\n- For other ambiguity, pick the most reasonable option and list it in the report.",
    verify: Some("- Run type check and build (`cargo check` or equivalent build command)\n- Run unit and integration tests (`cargo test --workspace -- --test-threads=1`)\n- Run linter and formatter check (`cargo clippy`, `cargo fmt --check`)"),
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. 🎯 **Key Changes**: (bullet points, max 1 line each)\n2. ⚠️ **Important / Blockers**: (assumptions, blockers, or critical decisions)\n3. 📋 **Remaining Tasks**: (what is left to do, actionable items)",
    out_of_scope: "Unrelated modules, breaking public API changes without prior approval, rewriting passing tests",
};

// --- DEBUG & BUG FIX ---
static DEBUG_BUGFIX_PROFILES: [CollectionProfile; 3] = [
    CollectionProfile::BugContext,
    CollectionProfile::TargetedFile,
    CollectionProfile::None,
];
static DEBUG_BUGFIX_FIELDS: [FieldContract; 5] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Bug description, symptom, or failure mechanism",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Observable criteria proving the bug is fixed and cannot recur",
    },
    FieldContract {
        name: "context",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Stack traces, observed error logs, or reproduction steps",
    },
    FieldContract {
        name: "relevant_files",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Target files containing the defect or directly related tests",
    },
    FieldContract {
        name: "target_test_command",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Command to reproduce or test the bug fix",
    },
];
static DEBUG_BUGFIX_STATIC: StaticSections = StaticSections {
    rules: "- Pinpoint root cause before modifying code; verify the failure mechanism first.\n- Minimal blast radius: touch only what is strictly necessary to resolve the bug.\n- Never suppress or swallow errors with empty catch blocks or loose typing.\n- Always add or update a regression test verifying the bug cannot recur.\n- Be EXTREMELY concise. The user skims text.",
    verify: Some("- Reproduce failure prior to fix to verify test failure\n- Run target regression test: `{{target_test_command}}`\n- Run full workspace test suite: `cargo test --workspace -- --test-threads=1`\n- Run linter: `cargo clippy --workspace --all-targets -- -D warnings`"),
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. 🔍 **Root Cause**: (exact line and failure mechanism)\n2. 🎯 **Fix Applied**: (specific files and logic changed)\n3. 🧪 **Verification Result**: (test command and pass confirmation)",
    out_of_scope: "Cosmetic refactorings, style changes in unrelated files, architecture redesigns",
};

// --- QUESTION & CONSULTATION ---
static QUESTION_PROFILES: [CollectionProfile; 3] = [
    CollectionProfile::TargetedFile,
    CollectionProfile::ArchitectureContext,
    CollectionProfile::None,
];
static QUESTION_FIELDS: [FieldContract; 4] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "The specific question or topic to be answered or explained",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Clear direct answer, verified code citations, and trade-off analysis",
    },
    FieldContract {
        name: "context",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Relevant subsystem, file pointer, or background context",
    },
    FieldContract {
        name: "relevant_files",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Specific files to inspect to answer the question",
    },
];
static QUESTION_STATIC: StaticSections = StaticSections {
    rules: "- Read-only analysis: do NOT make code edits or file modifications.\n- Ground all explanations strictly in the codebase; cite file paths and line numbers.\n- Anti-hallucination: if a fact is unverified or ambiguous, explicitly state it rather than assuming.\n- Be EXTREMELY concise; provide the direct answer first, followed by rationale.",
    verify: None,
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. 💡 **Direct Answer**: (core conclusion or answer)\n2. 📌 **Code References**: (file paths and line numbers)\n3. ⚖️ **Key Considerations / Trade-offs**: (if applicable)",
    out_of_scope: "Making code edits, modifying repository files, destructive commands",
};

// --- PLAN & ARCHITECTURE ---
static PLAN_PROFILES: [CollectionProfile; 3] = [
    CollectionProfile::ArchitectureContext,
    CollectionProfile::TargetedFile,
    CollectionProfile::None,
];
static PLAN_FIELDS: [FieldContract; 4] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Feature or architectural refactor goal to plan",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Structured, atomic, and testable phases starting with baseline",
    },
    FieldContract {
        name: "context",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Existing tech stack, module boundaries, and known constraints",
    },
    FieldContract {
        name: "relevant_files",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Key architecture files or interfaces in scope",
    },
];
static PLAN_STATIC: StaticSections = StaticSections {
    rules: "- Pure planning phase: DO NOT modify any code or configuration files.\n- Divide into small, atomic, independently testable phases (Step 0: Baseline, Step 1..N).\n- Every step must specify: Goal, files touched, verification command, and exit criteria.\n- Flag high-risk changes, database migrations, and public API breaking changes explicitly.\n- Ask before assuming architectural decisions or external dependencies.",
    verify: Some("- Verify all referenced files, structs, and functions exist in the repository\n- Validate feasibility and flags of proposed CLI commands against project environment"),
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. 🏛️ **Architecture / Plan Overview**: (1-2 sentences)\n2. 🪜 **Step-by-Step Breakdown**: (atomic phases with verification commands)\n3. ⚠️ **Risks & Open Questions**: (decisions needing user confirmation)",
    out_of_scope: "Writing production code or modifying files during this planning phase",
};

// --- TESTING & VERIFICATION ---
static TEST_PROFILES: [CollectionProfile; 3] = [
    CollectionProfile::TestContext,
    CollectionProfile::TargetedFile,
    CollectionProfile::None,
];
static TEST_FIELDS: [FieldContract; 5] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Target modules, functions, or workflows to test",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Happy path, error paths, and edge cases covered; 100% pass",
    },
    FieldContract {
        name: "context",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Testing framework, fixtures, or existing mocking utilities",
    },
    FieldContract {
        name: "relevant_files",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Target test files and tested implementation files",
    },
    FieldContract {
        name: "targeted_test_command",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Command to execute the new test suite",
    },
];
static TEST_STATIC: StaticSections = StaticSections {
    rules: "- Test real behavior; avoid brittle tautological assertions or mock-only tests.\n- Cover happy path, boundary values, error paths, and invalid inputs.\n- Do not alter or weaken existing passing tests to accommodate failures.\n- Deterministic tests only: no race conditions, sleep delays, or flaky external dependencies.\n- Be EXTREMELY concise.",
    verify: Some("- Run targeted test suite: `{{targeted_test_command}}`\n- Run full workspace tests: `cargo test --workspace -- --test-threads=1`\n- Run linter: `cargo clippy --workspace --all-targets -- -D warnings`"),
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. 🧪 **Tests Added / Updated**: (test functions and scenarios)\n2. ✅ **Test Execution Results**: (pass count, timing)\n3. 🛡️ **Edge Cases Covered**: (boundary and failure cases tested)",
    out_of_scope: "Modifying production business logic, weakening existing tests",
};

// --- CODEBASE AUDIT ---
static AUDIT_PROFILES: [CollectionProfile; 3] = [
    CollectionProfile::ArchitectureContext,
    CollectionProfile::TargetedFile,
    CollectionProfile::None,
];
static AUDIT_FIELDS: [FieldContract; 4] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Target scope or subsystem for codebase inspection",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Zero false-positive classification by severity with file:line citations",
    },
    FieldContract {
        name: "context",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Threat model, sensitive endpoints, or historical audit context",
    },
    FieldContract {
        name: "relevant_files",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Target paths or directories to inspect",
    },
];
static AUDIT_STATIC: StaticSections = StaticSections {
    rules: "- Read-only: DO NOT modify any code or files.\n- Zero false-positive policy: verify exploitability or reachability before reporting.\n- Classify findings by severity: Critical, High, Medium, Low, Info.\n- Include exact file paths and line ranges for every finding.\n- Be EXTREMELY concise; no fluff.",
    verify: Some("- Trace execution paths from public entry points to affected code to confirm reachability\n- Verify syntax and patterns with ripgrep/AST inspection"),
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. 🛡️ **Audit Findings**: (Severity, File:Line, Description)\n2. 💥 **Impact & Exploitability**: (realistic risk assessment)\n3. 🔧 **Remediation Steps**: (actionable recommendations for each finding)",
    out_of_scope: "Modifying files or applying code fixes during the audit phase",
};

// --- REPORT & SUMMARY ---
static REPORT_PROFILES: [CollectionProfile; 2] =
    [CollectionProfile::TargetedFile, CollectionProfile::None];
static REPORT_FIELDS: [FieldContract; 3] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Target milestone, feature status, or deliverable to report on",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Accurate reflection of codebase state, test results, and blockers",
    },
    FieldContract {
        name: "context",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Milestones, active issues, or timeframe covered",
    },
];
static REPORT_STATIC: StaticSections = StaticSections {
    rules: "- Read-only: do NOT make code edits or file modifications.\n- All status statements must be verified against actual files, git logs, or test runs.\n- No speculative or fabricated metrics.\n- Be EXTREMELY concise; use bullet points and tables.",
    verify: None,
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. 📊 **Progress & Current Status**: (factual completed deliverables)\n2. ⚠️ **Blockers & High-Priority Risks**: (active impediments)\n3. 📋 **Next Action Items**: (prioritized next steps)",
    out_of_scope: "Making code changes, committing files, mutating project state",
};

// --- RESEARCH & FEASIBILITY ---
static RESEARCH_PROFILES: [CollectionProfile; 3] = [
    CollectionProfile::ConfigurationContext,
    CollectionProfile::ArchitectureContext,
    CollectionProfile::None,
];
static RESEARCH_FIELDS: [FieldContract; 3] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Target library, tool, or approach to evaluate",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Evaluation matrix, license check, and clear go/no-go recommendation",
    },
    FieldContract {
        name: "context",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Current project stack, runtime constraints, and performance budgets",
    },
];
static RESEARCH_STATIC: StaticSections = StaticSections {
    rules: "- Read-only research: do not alter workspace files.\n- Evaluate compatibility with existing stack (Rust edition, OS support, licenses).\n- Distinguish verified capabilities from marketing claims.\n- Present trade-offs objectively; be EXTREMELY concise.",
    verify: Some("- Verify dependency compatibility with project dependencies and toolchain\n- Check open issue tracker and maintenance health of candidate libraries"),
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. ⚖️ **Evaluation Summary**: (options evaluated, pros & cons)\n2. 🎯 **Recommendation**: (chosen option with technical justification)\n3. ⚠️ **Compatibility & Risks**: (potential blockers, license traps)",
    out_of_scope: "Modifying repository code, installing untrusted dependencies, committing prototypes",
};

// --- DEVOPS & DEPLOYMENT ---
static DEVOPS_PROFILES: [CollectionProfile; 3] = [
    CollectionProfile::ConfigurationContext,
    CollectionProfile::TargetedFile,
    CollectionProfile::None,
];
static DEVOPS_FIELDS: [FieldContract; 5] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Target infrastructure, container, CI/CD, or deployment operation",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Idempotent operation, secrets protected, verification and rollback defined",
    },
    FieldContract {
        name: "context",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Target platform, orchestrator, and service topology",
    },
    FieldContract {
        name: "relevant_files",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Dockerfiles, manifests, or deployment scripts in scope",
    },
    FieldContract {
        name: "health_check_command",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Command to verify container or service health",
    },
];
static DEVOPS_STATIC: StaticSections = StaticSections {
    rules: "- NEVER commit, display, or log plaintext secrets, private keys, or credentials.\n- Dry-run and syntax check configurations before applying (`docker compose config`, `nginx -t`).\n- All operational scripts must be idempotent (safe to run repeatedly).\n- Always provide an immediate rollback procedure.\n- Be EXTREMELY concise.",
    verify: Some("- Run configuration syntax checks (`docker compose config`, `nginx -t`, etc.)\n- Test service build and check container health: `{{health_check_command}}`\n- Verify no plaintext secrets present in git diff: `git diff | grep -iE 'key|secret|password|token'`"),
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. 🚀 **Infrastructure Changes**: (containers, configs, scripts updated)\n2. 🔒 **Security & Secrets Check**: (confirmation no secrets leaked)\n3. 🔄 **Verification & Rollback**: (health check output and rollback command)",
    out_of_scope: "Committing plaintext secrets, mutating production database data directly",
};

// --- GIT & VCS ---
static GIT_PROFILES: [CollectionProfile; 1] = [CollectionProfile::None];
static GIT_FIELDS: [FieldContract; 4] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Git operation: commit changes, create branch, PR, or rebase",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Conventional commit format, clean working tree, no secrets staged",
    },
    FieldContract {
        name: "context",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Target branch and summary of intended changes",
    },
    FieldContract {
        name: "relevant_files",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Intended staged files",
    },
];
static GIT_STATIC: StaticSections = StaticSections {
    rules: "- Follow Conventional Commits format (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`).\n- Never force-push (`git push --force`) to main or protected branches.\n- Never commit untracked files without explicit verification.\n- Verify `git status` and staged diff before running commit.\n- Be EXTREMELY concise.",
    verify: Some("- Verify working tree status: `git status`\n- Verify latest commit formatting: `git log -1 --stat`"),
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. 📦 **Git Operation**: (branch name, commit hash, or PR status)\n2. 📝 **Commit Message / Summary**: (title and bullet points)\n3. 🔍 **Cleanliness Check**: (verification that no dirty/untracked files remain)",
    out_of_scope: "Committing untracked secrets, .env files, build artifacts (target/, node_modules/), force-pushing to main",
};

// --- CONFIG & TOOLING ---
static CONFIG_PROFILES: [CollectionProfile; 3] = [
    CollectionProfile::ConfigurationContext,
    CollectionProfile::TargetedFile,
    CollectionProfile::None,
];
static CONFIG_FIELDS: [FieldContract; 4] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Target CLI tool, runtime, or config setup to configure",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Installation verified, config syntax valid, existing comments preserved",
    },
    FieldContract {
        name: "relevant_files",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Target configuration files, dotfiles, or manifests",
    },
    FieldContract {
        name: "tool_command_check",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "CLI command to check tool version or help",
    },
];
static CONFIG_STATIC: StaticSections = StaticSections {
    rules: "- Preserve existing comments and unrelated settings in configuration files.\n- Respect OS-specific paths and conventions (macOS standards, ~/.config, LaunchAgents).\n- Validate syntax (JSON/TOML/YAML/plist) before saving.\n- No hardcoded machine-specific absolute paths unless mandatory.\n- Be EXTREMELY concise.",
    verify: Some("- Run tool version / help command: `{{tool_command_check}}`\n- Validate configuration file syntax: `{{syntax_validation_command}}`"),
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. ⚙️ **Configured Tools**: (tool name, version, status)\n2. 📂 **Files Modified**: (configuration file paths)\n3. 🧪 **Verification Output**: (output from CLI test command)",
    out_of_scope: "Modifying application source code or unrelated tool configurations",
};

// --- PROMPT ENGINEERING ---
static PROMPT_PROFILES: [CollectionProfile; 2] =
    [CollectionProfile::TargetedFile, CollectionProfile::None];
static PROMPT_FIELDS: [FieldContract; 4] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Target prompt, template, or formatting rule to create or enhance",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Adherence to XML schema, single-responsibility tags, token-efficient",
    },
    FieldContract {
        name: "context",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Target model capabilities, token budget, schema expectations",
    },
    FieldContract {
        name: "relevant_files",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Target prompt files or template definitions",
    },
];
static PROMPT_STATIC: StaticSections = StaticSections {
    rules: "- Single-responsibility tags: clear boundaries between task, criteria, rules, report.\n- Explicit anti-hallucination directives: instruct agent to acknowledge unknown or unverified data.\n- Token budget economy: keep instructions lean, avoiding conversational filler.\n- Proper XML/Markdown escaping and tag nesting.\n- Be EXTREMELY concise.",
    verify: Some("- Validate XML tag pairing and syntax\n- Verify token count fits within budget"),
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. 📐 **Prompt Architecture**: (tags included and rationale)\n2. 🎯 **Token Budget & Model Target**: (estimated token count)\n3. 🛡️ **Guardrails Enforced**: (anti-hallucination and security boundaries)",
    out_of_scope: "Modifying agent orchestrator logic or client code",
};

// --- DOCUMENTATION ---
static DOC_PROFILES: [CollectionProfile; 3] = [
    CollectionProfile::TargetedFile,
    CollectionProfile::ArchitectureContext,
    CollectionProfile::None,
];
static DOC_FIELDS: [FieldContract; 4] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Documentation or guidelines to author, update, or reorganize",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Codebase accuracy, verified links, and reproducible code snippets",
    },
    FieldContract {
        name: "context",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Related architecture, guidelines, or existing documentation",
    },
    FieldContract {
        name: "relevant_files",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Documentation files in scope for editing",
    },
];
static DOC_STATIC: StaticSections = StaticSections {
    rules: "- Documentation integrity: maintain existing docstrings and comments unless explicitly instructed.\n- Verify code snippets: all commands, flags, and code snippets in docs must be tested and accurate.\n- Keep docs concise, skim-friendly, and well-structured with GitHub Markdown.\n- Use clickable links with `file://` scheme for file and symbol references.\n- Be EXTREMELY concise.",
    verify: Some("- Verify all referenced file paths exist in repository\n- Execute documented CLI commands in a subshell to verify success"),
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. 📚 **Documentation Updated**: (files and sections modified)\n2. 🔗 **Verified References**: (code paths and commands verified)\n3. 💡 **Maintenance Notes**: (guidelines for future doc sync)",
    out_of_scope: "Modifying application source code or refactoring tests",
};

// --- SESSION PROGRESSION ---
static SESSION_PROFILES: [CollectionProfile; 3] = [
    CollectionProfile::TargetedFile,
    CollectionProfile::ArchitectureContext,
    CollectionProfile::None,
];
static SESSION_FIELDS: [FieldContract; 6] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Specific step or phase being approved and resumed from the plan",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Step exit check satisfied, baseline verified, step concluded",
    },
    FieldContract {
        name: "context",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Active plan reference, prior step completion status, and git state",
    },
    FieldContract {
        name: "relevant_files",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Files designated for this specific step",
    },
    FieldContract {
        name: "step_baseline_command",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Baseline verification command before modifying code",
    },
    FieldContract {
        name: "step_verification_command",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Exit check verification command for this step",
    },
];
static SESSION_STATIC: StaticSections = StaticSections {
    rules: "- Strictly execute ONLY the designated step; do NOT run ahead to subsequent steps.\n- Verify baseline before changing code; verify exit check before concluding.\n- Adhere strictly to the approved plan without unilateral deviations.\n- Be EXTREMELY concise. The user skims text.",
    verify: Some("- Run baseline check: `{{step_baseline_command}}`\n- Run step verification command: `{{step_verification_command}}`"),
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. 🪜 **Step Completed**: (step number and title)\n2. 🎯 **Key Changes Made**: (files and logic changes)\n3. ✅ **Exit Check Status**: (pass/fail against plan criteria)\n4. ⏩ **Ready for Next Step**: (next actionable step awaiting user cue)",
    out_of_scope: "Subsequent plan steps not yet approved, modifying files outside this step's scope",
};

// --- GENERAL TASK ---
static GENERAL_PROFILES: [CollectionProfile; 2] =
    [CollectionProfile::TargetedFile, CollectionProfile::None];
static GENERAL_FIELDS: [FieldContract; 4] = [
    FieldContract {
        name: "task",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "General instruction or maintenance objective",
    },
    FieldContract {
        name: "acceptance_criteria",
        required: true,
        evidence_policy: EvidencePolicy::UserTextOnly,
        description: "Observable, testable completion criteria",
    },
    FieldContract {
        name: "context",
        required: false,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Relevant background or repository context",
    },
    FieldContract {
        name: "relevant_files",
        required: true,
        evidence_policy: EvidencePolicy::EvidenceAllowed,
        description: "Files in scope for the task",
    },
];
static GENERAL_STATIC: StaticSections = StaticSections {
    rules: "- Be EXTREMELY concise. The user skims text. Do not explain how code works unless asked.\n- Follow existing patterns in the repo; check neighboring files first.\n- No mock data or swallowed errors; handle failures explicitly.\n- Ask before: destructive actions, schema or public API changes.\n- For other ambiguity, pick the most reasonable option and list it in the report.",
    verify: Some("- Run workspace test suite: `cargo test --workspace -- --test-threads=1`\n- Run linter: `cargo clippy --workspace --all-targets -- -D warnings`"),
    report: "KEEP YOUR RESPONSE EXTREMELY SHORT.\n1. 🎯 **Key Changes**: (bullet points, max 1 line each)\n2. ⚠️ **Important / Blockers**: (assumptions, blockers, or critical decisions)\n3. 📋 **Remaining Tasks**: (what is left to do, actionable items)",
    out_of_scope: "Unrelated modules, destructive changes without approval, breaking public APIs",
};

impl TemplateContract {
    pub fn for_template(id: TemplateId) -> &'static TemplateContract {
        match id {
            TemplateId::ImplementationRefactoring => &IMPL_REFACTOR_CONTRACT,
            TemplateId::DebugBugfix => &DEBUG_BUGFIX_CONTRACT,
            TemplateId::QuestionConsultation => &QUESTION_CONTRACT,
            TemplateId::PlanArchitecture => &PLAN_CONTRACT,
            TemplateId::TestingVerification => &TEST_CONTRACT,
            TemplateId::CodebaseAudit => &AUDIT_CONTRACT,
            TemplateId::ReportSummary => &REPORT_CONTRACT,
            TemplateId::ResearchFeasibility => &RESEARCH_CONTRACT,
            TemplateId::DevopsDeployment => &DEVOPS_CONTRACT,
            TemplateId::GitVcs => &GIT_CONTRACT,
            TemplateId::ConfigTooling => &CONFIG_CONTRACT,
            TemplateId::PromptEngineering => &PROMPT_CONTRACT,
            TemplateId::Documentation => &DOC_CONTRACT,
            TemplateId::SessionProgression => &SESSION_CONTRACT,
            TemplateId::GeneralTask => &GENERAL_CONTRACT,
        }
    }
}

pub static IMPL_REFACTOR_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::ImplementationRefactoring,
    session_kind: SessionKind::WriteEnabled,
    allowed_profiles: &IMPL_REFACTOR_PROFILES,
    dynamic_fields: &IMPL_REFACTOR_FIELDS,
    static_sections: IMPL_REFACTOR_STATIC,
};

pub static DEBUG_BUGFIX_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::DebugBugfix,
    session_kind: SessionKind::WriteEnabled,
    allowed_profiles: &DEBUG_BUGFIX_PROFILES,
    dynamic_fields: &DEBUG_BUGFIX_FIELDS,
    static_sections: DEBUG_BUGFIX_STATIC,
};

pub static QUESTION_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::QuestionConsultation,
    session_kind: SessionKind::ReadOnly,
    allowed_profiles: &QUESTION_PROFILES,
    dynamic_fields: &QUESTION_FIELDS,
    static_sections: QUESTION_STATIC,
};

pub static PLAN_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::PlanArchitecture,
    session_kind: SessionKind::ReadOnly,
    allowed_profiles: &PLAN_PROFILES,
    dynamic_fields: &PLAN_FIELDS,
    static_sections: PLAN_STATIC,
};

pub static TEST_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::TestingVerification,
    session_kind: SessionKind::WriteEnabled,
    allowed_profiles: &TEST_PROFILES,
    dynamic_fields: &TEST_FIELDS,
    static_sections: TEST_STATIC,
};

pub static AUDIT_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::CodebaseAudit,
    session_kind: SessionKind::ReadOnly,
    allowed_profiles: &AUDIT_PROFILES,
    dynamic_fields: &AUDIT_FIELDS,
    static_sections: AUDIT_STATIC,
};

pub static REPORT_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::ReportSummary,
    session_kind: SessionKind::ReadOnly,
    allowed_profiles: &REPORT_PROFILES,
    dynamic_fields: &REPORT_FIELDS,
    static_sections: REPORT_STATIC,
};

pub static RESEARCH_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::ResearchFeasibility,
    session_kind: SessionKind::ReadOnly,
    allowed_profiles: &RESEARCH_PROFILES,
    dynamic_fields: &RESEARCH_FIELDS,
    static_sections: RESEARCH_STATIC,
};

pub static DEVOPS_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::DevopsDeployment,
    session_kind: SessionKind::WriteEnabled,
    allowed_profiles: &DEVOPS_PROFILES,
    dynamic_fields: &DEVOPS_FIELDS,
    static_sections: DEVOPS_STATIC,
};

pub static GIT_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::GitVcs,
    session_kind: SessionKind::WriteEnabled,
    allowed_profiles: &GIT_PROFILES,
    dynamic_fields: &GIT_FIELDS,
    static_sections: GIT_STATIC,
};

pub static CONFIG_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::ConfigTooling,
    session_kind: SessionKind::WriteEnabled,
    allowed_profiles: &CONFIG_PROFILES,
    dynamic_fields: &CONFIG_FIELDS,
    static_sections: CONFIG_STATIC,
};

pub static PROMPT_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::PromptEngineering,
    session_kind: SessionKind::WriteEnabled,
    allowed_profiles: &PROMPT_PROFILES,
    dynamic_fields: &PROMPT_FIELDS,
    static_sections: PROMPT_STATIC,
};

pub static DOC_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::Documentation,
    session_kind: SessionKind::WriteEnabled,
    allowed_profiles: &DOC_PROFILES,
    dynamic_fields: &DOC_FIELDS,
    static_sections: DOC_STATIC,
};

pub static SESSION_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::SessionProgression,
    session_kind: SessionKind::WriteEnabled,
    allowed_profiles: &SESSION_PROFILES,
    dynamic_fields: &SESSION_FIELDS,
    static_sections: SESSION_STATIC,
};

pub static GENERAL_CONTRACT: TemplateContract = TemplateContract {
    template_id: TemplateId::GeneralTask,
    session_kind: SessionKind::WriteEnabled,
    allowed_profiles: &GENERAL_PROFILES,
    dynamic_fields: &GENERAL_FIELDS,
    static_sections: GENERAL_STATIC,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_15_templates_loadable() {
        for id in TemplateId::ALL {
            let contract = TemplateContract::for_template(id);
            assert_eq!(contract.template_id, id);
            assert!(!contract.dynamic_fields.is_empty());
            assert!(!contract.static_sections.rules.is_empty());
            assert!(!contract.static_sections.report.is_empty());
            assert!(!contract.static_sections.out_of_scope.is_empty());
        }
    }

    #[test]
    fn test_read_only_contracts_omit_or_have_no_verify() {
        let question = TemplateContract::for_template(TemplateId::QuestionConsultation);
        assert_eq!(question.static_sections.verify, None);
        assert_eq!(question.session_kind, SessionKind::ReadOnly);

        let report = TemplateContract::for_template(TemplateId::ReportSummary);
        assert_eq!(report.static_sections.verify, None);
        assert_eq!(report.session_kind, SessionKind::ReadOnly);
    }
}
