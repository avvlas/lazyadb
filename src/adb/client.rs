use std::process::Command;

use color_eyre::{eyre::eyre, Result};

use super::device::{parse_device_list, Device};

pub struct AdbClient {
    adb_path: String,
}

impl AdbClient {
    pub fn new() -> Result<Self> {
        let adb_path = std::env::var("ADB").unwrap_or_else(|_| "adb".to_string());

        let output = Command::new(&adb_path)
            .arg("version")
            .output()
            .map_err(|_| eyre!("Failed to run '{}'. Is adb installed and in PATH?", adb_path))?;

        if !output.status.success() {
            return Err(eyre!("'adb version' exited with non-zero status"));
        }

        Ok(Self { adb_path })
    }

    pub fn devices(&self) -> Result<Vec<Device>> {
        let output = Command::new(&self.adb_path)
            .args(["devices", "-l"])
            .output()
            .map_err(|e| eyre!("Failed to run 'adb devices -l': {}", e))?;

        if !output.status.success() {
            return Err(eyre!("'adb devices -l' failed"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_device_list(&stdout))
    }

    pub fn run_for_device(&self, serial: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(&self.adb_path)
            .arg("-s")
            .arg(serial)
            .args(args)
            .output()
            .map_err(|e| eyre!("Failed to run adb command: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(eyre!("adb command failed: {}", stderr.trim()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
