use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceState {
    Online,
    Offline,
    Unauthorized,
    Unknown(String),
}

impl fmt::Display for DeviceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Online => write!(f, "device"),
            Self::Offline => write!(f, "offline"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
    Usb,
    Tcp,
    Emulator,
}

#[derive(Debug, Clone)]
pub struct Device {
    pub serial: String,
    pub state: DeviceState,
    pub model: Option<String>,
    pub product: Option<String>,
    pub transport_id: Option<String>,
    pub connection_type: ConnectionType,
}

impl Device {
    pub fn display_name(&self) -> String {
        if let Some(model) = &self.model {
            model.replace('_', " ")
        } else {
            self.serial.clone()
        }
    }
}

pub fn parse_device_list(output: &str) -> Vec<Device> {
    let mut devices = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("List of") {
            continue;
        }

        let mut parts = line.split_whitespace();
        let serial = match parts.next() {
            Some(s) => s.to_string(),
            None => continue,
        };
        let state_str = match parts.next() {
            Some(s) => s,
            None => continue,
        };

        let state = match state_str {
            "device" => DeviceState::Online,
            "offline" => DeviceState::Offline,
            "unauthorized" => DeviceState::Unauthorized,
            other => DeviceState::Unknown(other.to_string()),
        };

        let connection_type = if serial.starts_with("emulator-") {
            ConnectionType::Emulator
        } else if serial.contains(':') {
            ConnectionType::Tcp
        } else {
            ConnectionType::Usb
        };

        let mut model = None;
        let mut product = None;
        let mut transport_id = None;

        for token in parts {
            if let Some((key, value)) = token.split_once(':') {
                match key {
                    "model" => model = Some(value.to_string()),
                    "product" => product = Some(value.to_string()),
                    "transport_id" => transport_id = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        devices.push(Device {
            serial,
            state,
            model,
            product,
            transport_id,
            connection_type,
        });
    }

    devices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_multi_device_output() {
        let output = "\
List of devices attached
ABCDEF1234     device usb:1-1 product:blueline model:Pixel_3 transport_id:1
192.168.1.100:5555 device product:generic model:SDK_Phone transport_id:2
emulator-5554  device product:sdk_phone model:sdk_phone transport_id:3

";
        let devices = parse_device_list(output);
        assert_eq!(devices.len(), 3);

        assert_eq!(devices[0].serial, "ABCDEF1234");
        assert_eq!(devices[0].state, DeviceState::Online);
        assert_eq!(devices[0].model.as_deref(), Some("Pixel_3"));
        assert_eq!(devices[0].connection_type, ConnectionType::Usb);
        assert_eq!(devices[0].display_name(), "Pixel 3");

        assert_eq!(devices[1].serial, "192.168.1.100:5555");
        assert_eq!(devices[1].connection_type, ConnectionType::Tcp);
        assert_eq!(devices[1].model.as_deref(), Some("SDK_Phone"));

        assert_eq!(devices[2].serial, "emulator-5554");
        assert_eq!(devices[2].connection_type, ConnectionType::Emulator);
    }

    #[test]
    fn parse_empty_output() {
        let output = "List of devices attached\n\n";
        let devices = parse_device_list(output);
        assert!(devices.is_empty());
    }

    #[test]
    fn parse_offline_and_unauthorized() {
        let output = "\
List of devices attached
OFFLINE123     offline transport_id:1
UNAUTH456      unauthorized transport_id:2
";
        let devices = parse_device_list(output);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].state, DeviceState::Offline);
        assert_eq!(devices[1].state, DeviceState::Unauthorized);
    }

    #[test]
    fn parse_unknown_state() {
        let output = "List of devices attached\nDEV001 recovery transport_id:1\n";
        let devices = parse_device_list(output);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].state, DeviceState::Unknown("recovery".into()));
    }

    #[test]
    fn display_name_falls_back_to_serial() {
        let device = Device {
            serial: "ABC123".into(),
            state: DeviceState::Online,
            model: None,
            product: None,
            transport_id: None,
            connection_type: ConnectionType::Usb,
        };
        assert_eq!(device.display_name(), "ABC123");
    }

    #[test]
    fn connection_type_detection() {
        let output = "\
List of devices attached
USB_DEVICE     device
192.168.1.1:5555 device
emulator-5554  device
";
        let devices = parse_device_list(output);
        assert_eq!(devices[0].connection_type, ConnectionType::Usb);
        assert_eq!(devices[1].connection_type, ConnectionType::Tcp);
        assert_eq!(devices[2].connection_type, ConnectionType::Emulator);
    }
}
