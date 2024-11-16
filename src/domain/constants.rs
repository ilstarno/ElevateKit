use winapi::shared::minwindef::DWORD;

pub const MAX_PRIVILEGES: usize = 36;

// Constants for Windows privileges and attributes
const SE_DELEGATE_SESSION_USER_IMPERSONATE_NAME: &str = "SeDelegateSessionUserImpersonatePrivilege";
const PROC_THREAD_ATTRIBUTE_PARENT_PROCESS: DWORD = 0x00020000;

// List of all Windows security privileges
pub const ALL_TOKEN_PRIVILEGES: [&str; MAX_PRIVILEGES] = [
    "SeAssignPrimaryTokenPrivilege",
    "SeAuditPrivilege",
    "SeBackupPrivilege",
    "SeChangeNotifyPrivilege",
    "SeCreateGlobalPrivilege",
    "SeCreatePagefilePrivilege",
    "SeCreatePermanentPrivilege",
    "SeCreateSymbolicLinkPrivilege",
    "SeCreateTokenPrivilege",
    "SeDebugPrivilege",
    SE_DELEGATE_SESSION_USER_IMPERSONATE_NAME,
    "SeEnableDelegationPrivilege",
    "SeImpersonatePrivilege",
    "SeIncreaseQuotaPrivilege",
    "SeIncreaseBasePriorityPrivilege",
    "SeIncreaseWorkingSetPrivilege",
    "SeLoadDriverPrivilege",
    "SeLockMemoryPrivilege",
    "SeMachineAccountPrivilege",
    "SeManageVolumePrivilege",
    "SeProfileSingleProcessPrivilege",
    "SeRelabelPrivilege",
    "SeRemoteShutdownPrivilege",
    "SeRestorePrivilege",
    "SeSecurityPrivilege",
    "SeShutdownPrivilege",
    "SeSyncAgentPrivilege",
    "SeSystemtimePrivilege",
    "SeSystemEnvironmentPrivilege",
    "SeSystemProfilePrivilege",
    "SeTakeOwnershipPrivilege",
    "SeTcbPrivilege",
    "SeTimeZonePrivilege",
    "SeTrustedCredManAccessPrivilege",
    "SeUndockPrivilege",
    "SeUnsolicitedInputPrivilege",
];


pub const SERVICE_NAME: &str = "TrustedInstaller";
pub const DEFAULT_SHELL: &str = "powershell.exe"; 