use std::fmt;

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub level: u8,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub used_gb: f64,
    pub total_gb: f64,
}

#[derive(Debug, Clone)]
pub struct RamInfo {
    pub used_gb: f64,
    pub total_gb: f64,
}

#[derive(Debug, Clone)]
pub struct ScreenInfo {
    pub resolution: String,
    pub density: String,
}

#[derive(Debug, Clone)]
pub struct WifiInfo {
    pub ssid: String,
    pub ip: String,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub serial: String,
    pub model: String,
    pub android_version: String,
    pub api_level: String,
    pub state: String,
    pub connection_type: String,
    pub abi: String,
    pub locale: String,
    pub battery: Option<BatteryInfo>,
    pub storage: Option<StorageInfo>,
    pub ram: Option<RamInfo>,
    pub screen: Option<ScreenInfo>,
    pub wifi: Option<WifiInfo>,
}

impl fmt::Display for BatteryInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}% ({})", self.level, self.status)
    }
}

impl fmt::Display for StorageInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}/{:.1} GB", self.used_gb, self.total_gb)
    }
}

impl fmt::Display for RamInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}/{:.1} GB", self.used_gb, self.total_gb)
    }
}

pub fn parse_getprop(output: &str) -> GetpropResult {
    let mut result = GetpropResult::default();
    for line in output.lines() {
        let line = line.trim();
        if let Some((key, value)) = parse_prop_line(line) {
            match key {
                "ro.build.version.release" => result.android_version = value.to_string(),
                "ro.build.version.sdk" => result.api_level = value.to_string(),
                "ro.product.cpu.abi" => result.abi = value.to_string(),
                "persist.sys.locale" => {
                    if result.locale.is_empty() {
                        result.locale = value.to_string();
                    }
                }
                "ro.product.locale" => {
                    if result.locale.is_empty() {
                        result.locale = value.to_string();
                    }
                }
                "ro.product.model" => result.model = value.to_string(),
                _ => {}
            }
        }
    }
    result
}

fn parse_prop_line(line: &str) -> Option<(&str, &str)> {
    // Format: [key]: [value]
    let line = line.strip_prefix('[')?;
    let (key, rest) = line.split_once("]:")?;
    let rest = rest.trim();
    let value = rest.strip_prefix('[')?.strip_suffix(']')?;
    Some((key, value))
}

#[derive(Debug, Default)]
pub struct GetpropResult {
    pub model: String,
    pub android_version: String,
    pub api_level: String,
    pub abi: String,
    pub locale: String,
}

pub fn parse_battery(output: &str) -> Option<BatteryInfo> {
    let mut level: Option<u8> = None;
    let mut status_code: Option<u32> = None;
    let mut plugged: Option<u32> = None;

    for line in output.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("level:") {
            level = val.trim().parse().ok();
        } else if let Some(val) = line.strip_prefix("status:") {
            status_code = val.trim().parse().ok();
        } else if let Some(val) = line.strip_prefix("plugged:") {
            plugged = val.trim().parse().ok();
        }
    }

    let level = level?;
    let status_str = match status_code.unwrap_or(0) {
        2 => {
            let plug_type = match plugged.unwrap_or(0) {
                1 => "AC",
                2 => "USB",
                4 => "Wireless",
                _ => "USB",
            };
            format!("charging ({})", plug_type)
        }
        3 => "discharging".to_string(),
        5 => "full".to_string(),
        4 => "not charging".to_string(),
        _ => "unknown".to_string(),
    };

    Some(BatteryInfo {
        level,
        status: status_str,
    })
}

pub fn parse_storage(output: &str) -> Option<StorageInfo> {
    // `df /data` output: Filesystem 1K-blocks Used Available Use% Mounted on
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let total_kb: f64 = parts[1].parse().ok()?;
            let used_kb: f64 = parts[2].parse().ok()?;
            return Some(StorageInfo {
                total_gb: total_kb / 1_048_576.0,
                used_gb: used_kb / 1_048_576.0,
            });
        }
    }
    None
}

pub fn parse_ram(output: &str) -> Option<RamInfo> {
    let mut total_kb: Option<f64> = None;
    let mut available_kb: Option<f64> = None;

    for line in output.lines() {
        if let Some(val) = line.strip_prefix("MemTotal:") {
            total_kb = val.trim().trim_end_matches(" kB").trim().parse().ok();
        } else if let Some(val) = line.strip_prefix("MemAvailable:") {
            available_kb = val.trim().trim_end_matches(" kB").trim().parse().ok();
        }
    }

    let total = total_kb?;
    let available = available_kb?;
    Some(RamInfo {
        total_gb: total / 1_048_576.0,
        used_gb: (total - available) / 1_048_576.0,
    })
}

pub fn parse_screen_size(output: &str) -> Option<String> {
    // "Physical size: 1080x2400"
    for line in output.lines() {
        if let Some(size) = line.strip_prefix("Physical size:") {
            let size = size.trim();
            return Some(size.replace('x', "\u{00d7}"));
        }
    }
    None
}

pub fn parse_screen_density(output: &str) -> Option<String> {
    // "Physical density: 420"
    for line in output.lines() {
        if let Some(density) = line.strip_prefix("Physical density:") {
            return Some(format!("{}dpi", density.trim()));
        }
    }
    None
}

pub fn parse_wifi(output: &str) -> Option<WifiInfo> {
    let mut ssid = None;
    let mut ip = None;

    for line in output.lines() {
        let trimmed = line.trim();
        if ssid.is_none() {
            if let Some(val) = trimmed.strip_prefix("mWifiInfo") {
                // Look for SSID in mWifiInfo line: SSID: "MyNetwork", ...
                if let Some(start) = val.find("SSID: ") {
                    let rest = &val[start + 6..];
                    let ssid_val = rest.split(',').next().unwrap_or("").trim();
                    let ssid_val = ssid_val.trim_matches('"');
                    if !ssid_val.is_empty() && ssid_val != "<unknown ssid>" {
                        ssid = Some(ssid_val.to_string());
                    }
                }
                if let Some(start) = val.find("IP: ") {
                    let rest = &val[start + 4..];
                    let ip_val = rest.split([',', '/']).next().unwrap_or("").trim();
                    if !ip_val.is_empty() && ip_val != "0.0.0.0" {
                        ip = Some(ip_val.to_string());
                    }
                }
            }
        }
    }

    Some(WifiInfo {
        ssid: ssid.unwrap_or_else(|| "N/A".to_string()),
        ip: ip.unwrap_or_else(|| "N/A".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_getprop() {
        let output = "\
[ro.build.version.release]: [14]
[ro.build.version.sdk]: [34]
[ro.product.cpu.abi]: [arm64-v8a]
[persist.sys.locale]: [en-US]
[ro.product.model]: [Pixel 7]
[some.other.prop]: [value]
";
        let result = parse_getprop(output);
        assert_eq!(result.android_version, "14");
        assert_eq!(result.api_level, "34");
        assert_eq!(result.abi, "arm64-v8a");
        assert_eq!(result.locale, "en-US");
        assert_eq!(result.model, "Pixel 7");
    }

    #[test]
    fn test_parse_battery() {
        let output = "\
Current Battery Service state:
  AC powered: false
  USB powered: true
  Wireless powered: false
  Max charging current: 500000
  status: 2
  health: 2
  present: true
  level: 72
  plugged: 2
  temperature: 250
";
        let info = parse_battery(output).unwrap();
        assert_eq!(info.level, 72);
        assert_eq!(info.status, "charging (USB)");
    }

    #[test]
    fn test_parse_battery_discharging() {
        let output = "  status: 3\n  level: 45\n  plugged: 0\n";
        let info = parse_battery(output).unwrap();
        assert_eq!(info.level, 45);
        assert_eq!(info.status, "discharging");
    }

    #[test]
    fn test_parse_storage() {
        let output = "\
Filesystem     1K-blocks    Used Available Use% Mounted on
/dev/block/dm-0 57542652 23456789 34085863  41% /data
";
        let info = parse_storage(output).unwrap();
        assert!((info.total_gb - 54.87).abs() < 0.1);
        assert!((info.used_gb - 22.37).abs() < 0.1);
    }

    #[test]
    fn test_parse_ram() {
        let output = "\
MemTotal:        7890000 kB
MemFree:          234000 kB
MemAvailable:    3456000 kB
Buffers:          123000 kB
";
        let info = parse_ram(output).unwrap();
        assert!((info.total_gb - 7.53).abs() < 0.1);
        assert!((info.used_gb - 4.23).abs() < 0.1);
    }

    #[test]
    fn test_parse_screen_size() {
        assert_eq!(
            parse_screen_size("Physical size: 1080x2400\n"),
            Some("1080\u{00d7}2400".to_string())
        );
    }

    #[test]
    fn test_parse_screen_density() {
        assert_eq!(
            parse_screen_density("Physical density: 420\n"),
            Some("420dpi".to_string())
        );
    }
}
