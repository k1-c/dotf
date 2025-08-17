use crate::cli::args::InstallTarget;
use crate::cli::Spinner;
use crate::core::{filesystem::RealFileSystem, scripts::SystemScriptExecutor};
use crate::error::DotfResult;
use crate::services::InstallService;
use crate::utils::ConsolePrompt;

pub async fn handle_install(target: InstallTarget) -> DotfResult<()> {
    let install_service = create_install_service();

    match target {
        InstallTarget::Deps => {
            let spinner = Spinner::new("Installing dependencies...");
            match install_service.install_dependencies().await {
                Ok(_) => spinner.finish_with_success("Dependencies installed successfully!"),
                Err(e) => {
                    spinner.finish_with_error(&format!("Dependencies installation failed: {}", e));
                    return Err(e);
                }
            }
        }
        InstallTarget::Config => {
            let spinner = Spinner::new("Installing configuration...");
            match install_service.install_config().await {
                Ok(_) => spinner.finish_with_success("Configuration installed successfully!"),
                Err(e) => {
                    spinner.finish_with_error(&format!("Configuration installation failed: {}", e));
                    return Err(e);
                }
            }
        }
        InstallTarget::Custom { name } => {
            let spinner = Spinner::new(&format!("Running custom script: {}", name));
            match install_service.install_custom(&name).await {
                Ok(_) => spinner.finish_with_success(&format!(
                    "Custom script '{}' completed successfully!",
                    name
                )),
                Err(e) => {
                    spinner.finish_with_error(&format!("Custom script '{}' failed: {}", name, e));
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

fn create_install_service() -> InstallService<RealFileSystem, SystemScriptExecutor, ConsolePrompt> {
    let filesystem = RealFileSystem::new();
    let script_executor = SystemScriptExecutor::new();
    let prompt = ConsolePrompt::new();

    InstallService::new(filesystem, script_executor, prompt)
}
