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

/// macOS virtual key code (`kVK_*`, as in `CGEventGetIntegerValueField(…, keycode)`) to a key
/// name. These are positions on the keyboard, not characters, so layouts don't matter.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn from_mac(code: u16) -> Option<&'static str> {
    Some(match code {
        0 => "KeyA",
        1 => "KeyS",
        2 => "KeyD",
        3 => "KeyF",
        4 => "KeyH",
        5 => "KeyG",
        6 => "KeyZ",
        7 => "KeyX",
        8 => "KeyC",
        9 => "KeyV",
        10 => "IntlBackslash",
        11 => "KeyB",
        12 => "KeyQ",
        13 => "KeyW",
        14 => "KeyE",
        15 => "KeyR",
        16 => "KeyY",
        17 => "KeyT",
        18 => "Digit1",
        19 => "Digit2",
        20 => "Digit3",
        21 => "Digit4",
        22 => "Digit6",
        23 => "Digit5",
        24 => "Equal",
        25 => "Digit9",
        26 => "Digit7",
        27 => "Minus",
        28 => "Digit8",
        29 => "Digit0",
        30 => "BracketRight",
        31 => "KeyO",
        32 => "KeyU",
        33 => "BracketLeft",
        34 => "KeyI",
        35 => "KeyP",
        36 => "Enter",
        37 => "KeyL",
        38 => "KeyJ",
        39 => "Quote",
        40 => "KeyK",
        41 => "Semicolon",
        42 => "Backslash",
        43 => "Comma",
        44 => "Slash",
        45 => "KeyN",
        46 => "KeyM",
        47 => "Period",
        48 => "Tab",
        49 => "Space",
        50 => "Backquote",
        51 => "Backspace",
        53 => "Escape",
        54 => "MetaRight",
        55 => "MetaLeft",
        56 => "ShiftLeft",
        57 => "CapsLock",
        58 => "AltLeft",
        59 => "ControlLeft",
        60 => "ShiftRight",
        61 => "AltRight",
        62 => "ControlRight",
        63 => "Fn",
        65 => "NumpadDecimal",
        67 => "NumpadMultiply",
        69 => "NumpadAdd",
        71 => "NumLock",
        75 => "NumpadDivide",
        76 => "NumpadEnter",
        78 => "NumpadSubtract",
        81 => "NumpadEqual",
        82 => "Numpad0",
        83 => "Numpad1",
        84 => "Numpad2",
        85 => "Numpad3",
        86 => "Numpad4",
        87 => "Numpad5",
        88 => "Numpad6",
        89 => "Numpad7",
        91 => "Numpad8",
        92 => "Numpad9",
        96 => "F5",
        97 => "F6",
        98 => "F7",
        99 => "F3",
        100 => "F8",
        101 => "F9",
        103 => "F11",
        105 => "F13",
        107 => "F14",
        109 => "F10",
        111 => "F12",
        113 => "F15",
        114 => "Insert",
        115 => "Home",
        116 => "PageUp",
        117 => "Delete",
        118 => "F4",
        119 => "End",
        120 => "F2",
        121 => "PageDown",
        122 => "F1",
        123 => "ArrowLeft",
        124 => "ArrowRight",
        125 => "ArrowDown",
        126 => "ArrowUp",
        _ => return None,
    })
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
        assert_eq!(from_mac(0), Some("KeyA"));
        assert_eq!(from_mac(49), Some("Space"));
        assert_eq!(from_mac(126), Some("ArrowUp"));
        assert_eq!(from_mac(55), Some("MetaLeft"));
    }
}
