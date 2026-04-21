use std::{
    env,
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

const COMMAND_TIMEOUT: Duration = Duration::from_millis(1500);
const COMMAND_POLL_STEP: Duration = Duration::from_millis(100);

pub(super) fn run_command_trimmed<const N: usize>(
    program: &str,
    args: [&str; N],
) -> Result<String, String> {
    let output = command_output_with_timeout(program, &args)
        .map_err(|error| format!("调用 {program} 失败：{error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        return Err(stderr
            .trim()
            .if_empty(stdout.trim())
            .if_empty("命令返回失败")
            .to_string());
    }

    Ok(stdout.lines().next().unwrap_or_default().trim().to_string())
}

pub(super) fn command_output_with_timeout(program: &str, args: &[&str]) -> Result<Output, String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().map_err(|error| error.to_string()),
            Ok(None) if start.elapsed() >= COMMAND_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "命令执行超时（>{}ms）",
                    COMMAND_TIMEOUT.as_millis()
                ));
            }
            Ok(None) => thread::sleep(COMMAND_POLL_STEP),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.to_string());
            }
        }
    }
}

pub(super) fn has_env(key: &str) -> bool {
    env::var(key)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

pub(super) trait EmptyFallback {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str;
}

impl EmptyFallback for str {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str {
        if self.trim().is_empty() {
            fallback
        } else {
            self
        }
    }
}
