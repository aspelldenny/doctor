//! cli subcmd dispatch — each subcmd in its own module.

pub mod lane_check;
pub mod validate_map;
pub mod rotate_check;
pub mod runtime_scan;
pub mod verify_setup;

/// Captured output from a subcmd execute() call.
///
/// exit_code: 0 = success, 1 = soft failure (budget/drift/leak), 2 = hard error (parse/IO)
/// stdout: lines that would have gone to stdout
/// stderr: lines that would have gone to stderr
#[derive(Debug, Default)]
pub struct RunOutput {
    pub exit_code: u8,
    pub stdout: String,
    pub stderr: String,
}
