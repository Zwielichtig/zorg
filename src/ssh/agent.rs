use std::process::Command;
use std::path::Path;

pub fn is_agent_running() -> bool {
    std::env::var("SSH_AUTH_SOCK").is_ok()
}

pub fn add_key_to_agent(key_path: &Path) -> std::io::Result<bool> {
    let status = Command::new("ssh-add")
        .arg(key_path)
        .status()?;
    Ok(status.success())
}

pub fn start_agent() -> std::io::Result<bool> {
    let output = Command::new("ssh-agent").arg("-s").output()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(kv) = line.split(';').next() {
                if let Some(pos) = kv.find('=') {
                    let key = &kv[..pos];
                    let val = &kv[pos+1..];
                    std::env::set_var(key, val);
                }
            }
        }
        Ok(true)
    } else {
        Ok(false)
    }
}
