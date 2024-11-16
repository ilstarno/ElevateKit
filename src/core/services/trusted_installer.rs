use crate::infrastructure::error::{TiError, TiResult};
use crate::infrastructure::windows::service::WindowsService;
use crate::infrastructure::windows::process::WindowsProcess;
use crate::application::services::privilege_service::PrivilegeService;
use log::{info, error};
use winapi::um::processthreadsapi::PROCESS_INFORMATION;
pub struct TrustedInstallerService {
    privilege_service: Box<dyn PrivilegeService>,
}

impl TrustedInstallerService {
    pub fn new(privilege_service: Box<dyn PrivilegeService>) -> Self {
        Self { privilege_service }
    }

    pub fn run_command(&self, command: &str) -> TiResult<()> {
        info!("Running command with TrustedInstaller privileges: {}", command);
        
        if command.trim().is_empty() {
            return Err(TiError::ProcessError("Command cannot be empty".into()));
        }

        let proc = self.create_elevated_process(command)?;
        self.privilege_service.set_all_privileges(proc.into())?;
        
        self.execute_and_wait(proc)
    }

    fn create_elevated_process(&self, command: &str) -> TiResult<PROCESS_INFORMATION> {
        // Get TrustedInstaller service PID
        let ti_pid = WindowsService::get_trusted_installer_pid()?;
        info!("TrustedInstaller service PID: {}", ti_pid);

        // Create process with TrustedInstaller privileges
        let process = WindowsProcess::new();
        let proc_info = process.create_process_as_trusted_installer(command, ti_pid)?;

        info!("Created elevated process with PID: {}", proc_info.dwProcessId);
        Ok(proc_info)
    }

    fn execute_and_wait(&self, proc: PROCESS_INFORMATION) -> TiResult<()> {
        let process = WindowsProcess::new();
        
        // Resume the process
        info!("Resuming process");
        process.resume_process(&proc)?;

        // Wait for process to exit and get exit code
        info!("Waiting for process to exit");
        let exit_code = process.wait_for_exit(&proc)?;

        if exit_code != 0 {
            error!("Process exited with code {}", exit_code);
            return Err(TiError::ProcessError(format!("Process exited with code {}", exit_code)));
        }

        info!("Process completed successfully");
        Ok(())
    }
} 