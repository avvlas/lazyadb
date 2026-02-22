use std::path::PathBuf;
use std::process::{Command, Stdio};

use color_eyre::{Result, eyre::eyre};
use tracing::info;

use super::device::{ConnectionType, Device, parse_device_list};
use super::device_info::*;
use super::emulator::{Avd, parse_avd_list};

pub struct AdbClient {
    adb_path: String,
    emulator_path: String,
}

impl AdbClient {
    pub fn new() -> Result<Self> {
        let adb_path = std::env::var("ADB").unwrap_or_else(|_| "adb".to_string());
        let emulator_path = resolve_emulator_path();

        let output = Command::new(&adb_path)
            .arg("version")
            .output()
            .map_err(|_| {
                eyre!(
                    "Failed to run '{}'. Is adb installed and in PATH?",
                    adb_path
                )
            })?;

        if !output.status.success() {
            return Err(eyre!("'adb version' exited with non-zero status"));
        }

        info!(emulator_path = %emulator_path, "AdbClient initialized");

        Ok(Self {
            adb_path,
            emulator_path,
        })
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

    pub fn list_avds(&self) -> Result<Vec<String>> {
        let output = Command::new(&self.emulator_path)
            .arg("-list-avds")
            .output()
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to run 'emulator -list-avds'");
                eyre!("Failed to run 'emulator -list-avds': {}", e)
            })?;

        if !output.status.success() {
            return Err(eyre!("'emulator -list-avds' failed"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let avds = parse_avd_list(&stdout);
        info!(avds = ?avds, "Listed AVDs");
        Ok(avds)
    }

    pub fn get_avd_name(&self, serial: &str) -> Option<String> {
        let output = Command::new(&self.adb_path)
            .args(["-s", serial, "emu", "avd", "name"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.lines().next().map(|l| l.trim().to_string())
    }

    pub fn avds_with_status(&self, devices: &[Device]) -> Vec<Avd> {
        let avd_names = self.list_avds().unwrap_or_default();

        let running_emulators: Vec<&Device> = devices
            .iter()
            .filter(|d| d.connection_type == ConnectionType::Emulator)
            .collect();

        let mut serial_to_avd: Vec<(String, String)> = Vec::new();
        for emu in &running_emulators {
            if let Some(name) = self.get_avd_name(&emu.serial) {
                serial_to_avd.push((emu.serial.clone(), name));
            }
        }

        let mut avds: Vec<Avd> = avd_names
            .into_iter()
            .map(|name| {
                let running_serial = serial_to_avd
                    .iter()
                    .find(|(_, avd_name)| avd_name == &name)
                    .map(|(serial, _)| serial.clone());
                Avd {
                    name,
                    running_serial,
                }
            })
            .collect();

        // Add running emulators not in the AVD list (e.g. started outside)
        for (serial, avd_name) in &serial_to_avd {
            if !avds.iter().any(|a| &a.name == avd_name) {
                avds.push(Avd {
                    name: avd_name.clone(),
                    running_serial: Some(serial.clone()),
                });
            }
        }

        avds
    }

    pub fn start_emulator(&self, avd_name: &str) -> Result<()> {
        Command::new(&self.emulator_path)
            .args(["-avd", avd_name])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| eyre!("Failed to start emulator '{}': {}", avd_name, e))?;
        Ok(())
    }

    pub fn kill_emulator(&self, serial: &str) -> Result<String> {
        self.run_for_device(serial, &["emu", "kill"])
    }

    pub fn fetch_device_info(&self, device: &Device) -> Result<DeviceInfo> {
        let serial = &device.serial;

        // Batch property read
        let props_output = self.run_for_device(serial, &["shell", "getprop"]).unwrap_or_default();
        let props = parse_getprop(&props_output);

        // Battery
        let battery = self
            .run_for_device(serial, &["shell", "dumpsys", "battery"])
            .ok()
            .and_then(|out| parse_battery(&out));

        // Storage
        let storage = self
            .run_for_device(serial, &["shell", "df", "/data"])
            .ok()
            .and_then(|out| parse_storage(&out));

        // RAM
        let ram = self
            .run_for_device(serial, &["shell", "cat", "/proc/meminfo"])
            .ok()
            .and_then(|out| parse_ram(&out));

        // Screen
        let screen = {
            let size = self
                .run_for_device(serial, &["shell", "wm", "size"])
                .ok()
                .and_then(|out| parse_screen_size(&out));
            let density = self
                .run_for_device(serial, &["shell", "wm", "density"])
                .ok()
                .and_then(|out| parse_screen_density(&out));
            match (size, density) {
                (Some(res), Some(den)) => Some(ScreenInfo {
                    resolution: res,
                    density: den,
                }),
                (Some(res), None) => Some(ScreenInfo {
                    resolution: res,
                    density: "N/A".to_string(),
                }),
                _ => None,
            }
        };

        // Wi-Fi
        let wifi = self
            .run_for_device(serial, &["shell", "dumpsys", "wifi"])
            .ok()
            .and_then(|out| parse_wifi(&out));

        let model = if !props.model.is_empty() {
            props.model
        } else {
            device.display_name()
        };

        let conn_type = match device.connection_type {
            ConnectionType::Usb => "USB",
            ConnectionType::Tcp => "TCP",
            ConnectionType::Emulator => "Emulator",
        };

        Ok(DeviceInfo {
            serial: serial.clone(),
            model,
            android_version: if props.android_version.is_empty() {
                "N/A".to_string()
            } else {
                props.android_version
            },
            api_level: if props.api_level.is_empty() {
                "N/A".to_string()
            } else {
                props.api_level
            },
            state: device.state.to_string(),
            connection_type: conn_type.to_string(),
            abi: if props.abi.is_empty() {
                "N/A".to_string()
            } else {
                props.abi
            },
            locale: if props.locale.is_empty() {
                "N/A".to_string()
            } else {
                props.locale
            },
            battery,
            storage,
            ram,
            screen,
            wifi,
        })
    }

    pub fn disconnect_device(&self, serial: &str) -> Result<()> {
        let output = Command::new(&self.adb_path)
            .args(["disconnect", serial])
            .output()
            .map_err(|e| eyre!("Failed to run 'adb disconnect {}': {}", serial, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(eyre!(
                "'adb disconnect {}' failed: {}",
                serial,
                stderr.trim()
            ));
        }

        Ok(())
    }
}

fn resolve_emulator_path() -> String {
    // 1. Try ANDROID_HOME or ANDROID_SDK_ROOT
    let sdk_dir = std::env::var("ANDROID_HOME")
        .or_else(|_| std::env::var("ANDROID_SDK_ROOT"))
        .ok();

    if let Some(sdk) = sdk_dir {
        let candidate = PathBuf::from(&sdk).join("emulator").join("emulator");
        if candidate.exists() {
            return candidate.to_string_lossy().into_owned();
        }
    }

    // 2. Fall back to bare name (hope it's in PATH)
    "emulator".to_string()
}
