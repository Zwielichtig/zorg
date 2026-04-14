use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
pub struct SshKeyInfo {
    pub path: PathBuf,
    pub is_private: bool,
    pub has_secure_permissions: bool,
}

pub fn get_available_keys() -> Vec<SshKeyInfo> {
    let mut keys = Vec::new();
    let home = std::env::var("HOME").unwrap_or_else(|_| "".to_string());
    if home.is_empty() {
        return keys;
    }
    
    let ssh_dir = Path::new(&home).join(".ssh");
    if !ssh_dir.exists() {
        return keys;
    }

    if let Ok(entries) = fs::read_dir(ssh_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                if file_name == "config" || file_name == "known_hosts" || file_name.ends_with(".known_hosts") || file_name.ends_with(".pub") {
                    continue;
                }
                
                let is_private = !file_name.ends_with(".pub");
                let mut has_secure_permissions = true;
                
                if is_private {
                    if let Ok(metadata) = fs::metadata(&path) {
                        let mode = metadata.permissions().mode();
                        if mode & 0o077 != 0 {
                            has_secure_permissions = false;
                        }
                    }
                }
                
                keys.push(SshKeyInfo {
                    path,
                    is_private,
                    has_secure_permissions,
                });
            }
        }
    }
    keys
}

