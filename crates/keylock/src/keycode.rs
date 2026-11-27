//! Physical key names, as W3C `KeyboardEvent.code` strings ("KeyA", "Space", "ArrowUp").
//!
//! Every platform reports keys in its own numbering. They all meet here, so the sound engine
//! (and Mechvibes sound packs, which use these names) never sees an OS key code.

/// PC set-1 scan code (what Windows reports, and what old Mechvibes packs and libuiohook use)
/// to a key name. `extended` is the E0 prefix. Unknown keys give `None`.
pub fn from_set1(scan: u16, extended: bool) -> Option<&'static str> {
    if extended {
        return Some(match scan {
            0x1C => "NumpadEnter",
            0x1D => "ControlRight",
            0x35 => "NumpadDivide",
            0x36 => "ShiftRight",
            0x37 => "PrintScreen",
            0x38 => "AltRight",
            0x45 => "NumLock",
            0x46 => "Pause",
            0x47 => "Home",
            0x48 => "ArrowUp",
            0x49 => "PageUp",
            0x4B => "ArrowLeft",
            0x4D => "ArrowRight",
            0x4F => "End",
            0x50 => "ArrowDown",
            0x51 => "PageDown",
            0x52 => "Insert",
            0x53 => "Delete",
            0x5B => "MetaLeft",
            0x5C => "MetaRight",
            0x5D => "ContextMenu",
            _ => return None,
        });
    }
    const ROW: [&str; 0x59] = [
        "", "Escape", "Digit1", "Digit2", "Digit3", "Digit4", "Digit5", "Digit6", // 0x00
        "Digit7", "Digit8", "Digit9", "Digit0", "Minus", "Equal", "Backspace", "Tab", // 0x08
        "KeyQ", "KeyW", "KeyE", "KeyR", "KeyT", "KeyY", "KeyU", "KeyI", // 0x10
        "KeyO", "KeyP", "BracketLeft", "BracketRight", "Enter", "ControlLeft", "KeyA", "KeyS", // 0x18
        "KeyD", "KeyF", "KeyG", "KeyH", "KeyJ", "KeyK", "KeyL", "Semicolon", // 0x20
        "Quote", "Backquote", "ShiftLeft", "Backslash", "KeyZ", "KeyX", "KeyC", "KeyV", // 0x28
        "KeyB", "KeyN", "KeyM", "Comma", "Period", "Slash", "ShiftRight", "NumpadMultiply", // 0x30
        "AltLeft", "Space", "CapsLock", "F1", "F2", "F3", "F4", "F5", // 0x38
        "F6", "F7", "F8", "F9", "F10", "NumLock", "ScrollLock", "Numpad7", // 0x40
        "Numpad8", "Numpad9", "NumpadSubtract", "Numpad4", "Numpad5", "Numpad6", "NumpadAdd", "Numpad1", // 0x48
        "Numpad2", "Numpad3", "Numpad0", "NumpadDecimal", "", "", "IntlBackslash", "F11", // 0x50
        "F12", // 0x58
    ];
    ROW.get(scan as usize).copied().filter(|s| !s.is_empty())
}

/// A legacy (V1) Mechvibes / libuiohook key code: the set-1 scan code in the low byte, with
/// 0x0E00 or 0xE000 marking the extended keys.
pub fn from_uiohook(code: u32) -> Option<&'static str> {
    from_set1((code & 0xFF) as u16, code & 0xFF00 != 0)
}

/// Linux evdev key code (`KEY_*`) to a key name. The basic block matches set 1.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn from_evdev(code: u16) -> Option<&'static str> {
    match code {
        1..=88 => from_set1(code, false),
        96 => Some("NumpadEnter"),
        97 => Some("ControlRight"),
        98 => Some("NumpadDivide"),
        99 => Some("PrintScreen"),
        100 => Some("AltRight"),
        102 => Some("Home"),
        103 => Some("ArrowUp"),
        104 => Some("PageUp"),
        105 => Some("ArrowLeft"),
        106 => Some("ArrowRight"),
        107 => Some("End"),
        108 => Some("ArrowDown"),
        109 => Some("PageDown"),
        110 => Some("Insert"),
        111 => Some("Delete"),
        119 => Some("Pause"),
        125 => Some("MetaLeft"),
        126 => Some("MetaRight"),
        127 => Some("ContextMenu"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_common_keys() {
        assert_eq!(from_set1(0x1E, false), Some("KeyA"));
        assert_eq!(from_set1(0x39, false), Some("Space"));
        assert_eq!(from_set1(0x48, true), Some("ArrowUp"));
        assert_eq!(from_set1(0x48, false), Some("Numpad8"));
        assert_eq!(from_uiohook(57416), Some("ArrowUp"));
        assert_eq!(from_uiohook(3612), Some("NumpadEnter"));
        assert_eq!(from_evdev(57), Some("Space"));
        assert_eq!(from_evdev(125), Some("MetaLeft"));
        assert_eq!(from_set1(0x54, false), None);
    }
}
