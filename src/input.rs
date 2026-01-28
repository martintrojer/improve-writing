use anyhow::{Result, anyhow};
use evdev::{Device, Key};

#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

#[derive(Debug, Clone)]
pub struct Hotkey {
    pub key: Key,
    pub modifiers: Modifiers,
}

/// Parse a hotkey string like "Shift+F9" or "F10" into a Hotkey
pub fn parse_hotkey(s: &str) -> Result<Hotkey> {
    let parts: Vec<&str> = s.split('+').collect();
    let mut modifiers = Modifiers::default();
    let key_str;

    if parts.len() == 1 {
        key_str = parts[0];
    } else {
        // Parse modifiers
        for part in &parts[..parts.len() - 1] {
            match part.to_uppercase().as_str() {
                "SHIFT" => modifiers.shift = true,
                "CTRL" | "CONTROL" => modifiers.ctrl = true,
                "ALT" => modifiers.alt = true,
                _ => return Err(anyhow!("Unknown modifier: {}", part)),
            }
        }
        key_str = parts[parts.len() - 1];
    }

    let key = match key_str.to_uppercase().as_str() {
        "F1" => Key::KEY_F1,
        "F2" => Key::KEY_F2,
        "F3" => Key::KEY_F3,
        "F4" => Key::KEY_F4,
        "F5" => Key::KEY_F5,
        "F6" => Key::KEY_F6,
        "F7" => Key::KEY_F7,
        "F8" => Key::KEY_F8,
        "F9" => Key::KEY_F9,
        "F10" => Key::KEY_F10,
        "F11" => Key::KEY_F11,
        "F12" => Key::KEY_F12,
        "SCROLLLOCK" => Key::KEY_SCROLLLOCK,
        "PAUSE" => Key::KEY_PAUSE,
        "INSERT" => Key::KEY_INSERT,
        _ => return Err(anyhow!("Unknown key: {}", key_str)),
    };

    Ok(Hotkey { key, modifiers })
}

/// Find all keyboard devices
pub fn find_keyboards() -> Result<Vec<Device>> {
    let mut keyboards = Vec::new();

    for entry in std::fs::read_dir("/dev/input")? {
        let entry = entry?;
        let path = entry.path();

        if !path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("event"))
            .unwrap_or(false)
        {
            continue;
        }

        if let Ok(device) = Device::open(&path) {
            // Check if device supports keyboard keys
            if device
                .supported_keys()
                .map(|keys| keys.contains(Key::KEY_A))
                .unwrap_or(false)
            {
                log::debug!("Found keyboard: {:?} at {:?}", device.name(), path);
                keyboards.push(device);
            }
        }
    }

    if keyboards.is_empty() {
        Err(anyhow!(
            "No keyboards found. Make sure you're in the 'input' group or running as root."
        ))
    } else {
        Ok(keyboards)
    }
}
