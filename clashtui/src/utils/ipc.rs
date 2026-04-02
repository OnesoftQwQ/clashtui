use std::process::{Command, Output, Stdio};

use std::io::{Result, Write};

pub fn exec(pgm: &str, args: Vec<&str>) -> Result<String> {
    log::debug!("IPC: {} {:?}", pgm, args);
    let output = Command::new(pgm).args(args).output()?;
    string_process_output(output)
}

pub fn spawn(pgm: &str, args: Vec<&str>) -> Result<()> {
    log::debug!("SPW: {} {:?}", pgm, args);
    // Just ignore the output, otherwise the ui might be broken
    Command::new(pgm)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .args(args)
        .spawn()?;
    Ok(())
}

pub fn exec_with_sbin(pgm: &str, args: Vec<&str>) -> Result<String> {
    log::debug!("LIPC: {} {:?}", pgm, args);
    let mut path = std::env::var("PATH").unwrap_or_default();
    path.push_str(":/usr/sbin");
    let output = Command::new(pgm).env("PATH", path).args(args).output()?;
    string_process_output(output)
}

fn string_process_output(output: Output) -> Result<String> {
    let stdout_str = String::from_utf8(output.stdout).unwrap();
    let stderr_str = String::from_utf8(output.stderr).unwrap();

    let result_str = format!(
        r#"
        Status:
        {}

        Stdout:
        {}

        Stderr:
        {}
        "#,
        output.status, stdout_str, stderr_str
    );

    Ok(result_str)
}

// Returns true if sudo password is required, false otherwise
pub fn check_sudo_password_required() -> bool {
    let output = Command::new("sudo")
        .arg("-n")
        .arg("true")
        .output();

    match output {
        Ok(output) => {
            !output.status.success()
        }
        Err(e) => {
            log::error!("`sudo -n true`: {e}");
            true
        }
    }
}

pub fn exec_with_password(pgm: &str, args: Vec<&str>, passwd: &str) -> Result<String> {
    let mut child = Command::new("sudo")
        .arg("-S")
        .arg("-p")
        .arg("")
        .arg("--")
        .arg(pgm)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(passwd.as_bytes())?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;
    }

    let output = child.wait_with_output()?;
    
    string_process_output(output)
}