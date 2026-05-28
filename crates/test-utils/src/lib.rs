use std::process::Command;

/// Extension trait for [`Command`] that runs the process and asserts the outcome,
/// returning the captured output stream as a `String`.
pub trait RunExt {
    /// Runs the command, asserts it exited successfully with empty stderr,
    /// and returns stdout.
    fn run_ok(&mut self) -> String;

    /// Runs the command, asserts it exited unsuccessfully with empty stdout,
    /// and returns stderr.
    fn run_err(&mut self) -> String;
}

impl RunExt for Command {
    fn run_ok(&mut self) -> String {
        let output = self.output().unwrap();
        assert!(
            output.status.success(),
            "expected success, got stderr: {}",
            String::from_utf8_lossy(&output.stderr),
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
        String::from_utf8(output.stdout).unwrap()
    }

    fn run_err(&mut self) -> String {
        let output = self.output().unwrap();
        assert!(
            !output.status.success(),
            "expected failure, got stdout: {}",
            String::from_utf8_lossy(&output.stdout),
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.is_empty(), "unexpected stdout: {stdout}");
        String::from_utf8(output.stderr).unwrap()
    }
}
