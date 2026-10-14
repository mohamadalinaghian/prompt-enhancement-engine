use clap::Parser;
use pe_cli::{execute_pipeline, SingleFlightLock};
use pe_core::config::Config;
use pe_core::TerminalStatus;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = "pe",
    about = "Prompt Enhancement Engine — high-performance prompt structuring utility for macOS",
    version
)]
struct Args {
    /// Optional custom configuration file path
    #[arg(long)]
    config: Option<PathBuf>,

    /// Force deterministic offline path (no external LLM/Jev network calls)
    #[arg(long)]
    no_llm: bool,

    /// Optional custom lockfile path
    #[arg(long)]
    lock_path: Option<PathBuf>,

    /// Target application identity passed by shell integration
    #[arg(long, default_value = "unknown")]
    app: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // 1. Acquire Single-Flight Runtime Lock (D-014, exit code 3 if busy)
    let _lock = match SingleFlightLock::acquire(args.lock_path.as_deref()) {
        Ok(l) => l,
        Err(status) => {
            eprintln!("{}", status);
            exit(status.exit_code());
        }
    };

    // 2. Load Configuration
    let config = Config::load_or_default(args.config.as_deref());

    // 3. Read Raw Input from stdin
    let mut raw_input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut raw_input) {
        eprintln!("INPUT_EMPTY failed to read stdin: {}", e);
        exit(TerminalStatus::InputEmpty.exit_code());
    }

    // 4. Overall Deadline Supervisor (D-015, provisional 10,000 ms)
    let deadline = Duration::from_millis(config.budgets.total_deadline_ms);

    let outcome_result =
        tokio::time::timeout(deadline, execute_pipeline(&raw_input, &config, args.no_llm)).await;

    let outcome = match outcome_result {
        Ok(res) => res,
        Err(_) => {
            // Overall deadline exceeded
            eprintln!("{}", TerminalStatus::DeadlineExceeded);
            exit(TerminalStatus::DeadlineExceeded.exit_code());
        }
    };

    // 5. Output Machine-Readable Status to stderr
    eprintln!("{}", outcome.flags.format_stderr(outcome.status));

    // 6. Output Final Payload to stdout if successful or clarification
    if let Some(payload) = outcome.output_payload {
        print!("{}", payload);
    }

    // 7. Exit with Status-Specific Exit Code (workflow-integration.md §4)
    exit(outcome.status.exit_code());
}
