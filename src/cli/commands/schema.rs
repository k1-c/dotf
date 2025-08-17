use crate::cli::args::SchemaAction;
use crate::error::DotfResult;
use crate::services::{SchemaService, SchemaValidator};
use std::process;

pub async fn handle_schema(action: SchemaAction) -> DotfResult<()> {
    match action {
        SchemaAction::Init => handle_schema_init().await,
        SchemaAction::Test {
            file,
            ignore_errors,
            quiet,
        } => handle_schema_test(file, ignore_errors, quiet).await,
    }
}

async fn handle_schema_init() -> DotfResult<()> {
    let service = SchemaService::new();
    service.init().await
}

async fn handle_schema_test(
    file: Option<String>,
    ignore_errors: bool,
    quiet: bool,
) -> DotfResult<()> {
    let validator = SchemaValidator::new();
    let file_path = file.unwrap_or_else(|| "dotf.toml".to_string());

    match validator.validate(&file_path).await {
        Ok(result) => {
            let output = validator.format_result(&result, quiet);
            println!("{}", output);

            if !result.is_valid && !ignore_errors {
                process::exit(1);
            }
            Ok(())
        }
        Err(e) => {
            if !quiet {
                eprintln!("🚨 Validation failed: {}", e);
            } else {
                eprintln!("Error: {}", e);
            }

            if !ignore_errors {
                process::exit(2);
            }
            Ok(())
        }
    }
}
