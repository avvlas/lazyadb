#[derive(Debug, Clone, PartialEq)]
pub struct Avd {
    pub name: String,
    pub running_serial: Option<String>,
}

impl Avd {
    pub fn is_running(&self) -> bool {
        self.running_serial.is_some()
    }

    pub fn display_name(&self) -> String {
        self.name.replace('_', " ")
    }
}

pub fn parse_avd_list(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_avd_list_basic() {
        let output = "Pixel_4_API_30\nPixel_6_Pro_API_33\n";
        let avds = parse_avd_list(output);
        assert_eq!(avds, vec!["Pixel_4_API_30", "Pixel_6_Pro_API_33"]);
    }

    #[test]
    fn parse_avd_list_empty() {
        let avds = parse_avd_list("");
        assert!(avds.is_empty());
    }

    #[test]
    fn parse_avd_list_with_blank_lines() {
        let output = "\n  Pixel_4_API_30  \n\n  Pixel_6_Pro_API_33\n\n";
        let avds = parse_avd_list(output);
        assert_eq!(avds, vec!["Pixel_4_API_30", "Pixel_6_Pro_API_33"]);
    }

    #[test]
    fn display_name_replaces_underscores() {
        let avd = Avd {
            name: "Pixel_4_API_30".to_string(),
            running_serial: None,
        };
        assert_eq!(avd.display_name(), "Pixel 4 API 30");
    }

    #[test]
    fn is_running_checks_serial() {
        let stopped = Avd {
            name: "Test".to_string(),
            running_serial: None,
        };
        assert!(!stopped.is_running());

        let running = Avd {
            name: "Test".to_string(),
            running_serial: Some("emulator-5554".to_string()),
        };
        assert!(running.is_running());
    }
}
