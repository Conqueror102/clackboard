//! Mechvibes sound packs: a folder with `config.json` and one or more audio files.
//!
//! Both formats are read:
//! * V2 (current Mechvibes): `definitions` maps W3C key names to `{ "timing": [[start, end], …] }`
//!   in milliseconds. One segment is the press; a second, when present, is the release.
//! * V1 (the thousands of community packs): `defines` maps libuiohook key codes to
//!   `[start, duration]` (one file, `key_define_type: "single"`) or to a file name ("multi").

use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Decoded audio, stereo interleaved f32 at the file's own rate.
pub struct Buffer {
    pub samples: Vec<f32>,
    pub rate: u32,
}

impl Buffer {
    pub fn frames(&self) -> usize {
        self.samples.len() / 2
    }
}

/// A slice of a buffer, in frames.
#[derive(Clone)]
pub struct Segment {
    pub buf: Arc<Buffer>,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone)]
pub struct KeySound {
    pub down: Segment,
    pub up: Option<Segment>,
}

pub struct Pack {
    pub keys: HashMap<String, KeySound>,
    /// Sorted key names, for a stable fallback when a key has no sound of its own.
    names: Vec<String>,
    pub gain: f32,
}

impl Pack {
    pub fn sound_for(&self, code: &str) -> Option<&KeySound> {
        if let Some(s) = self.keys.get(code) {
            return Some(s);
        }
        if self.names.is_empty() {
            return None;
        }
        // Same key, same stand-in sound, so an unknown key still feels like one key.
        let h = code.bytes().fold(5381u32, |h, b| h.wrapping_mul(33) ^ b as u32);
        self.keys.get(&self.names[h as usize % self.names.len()])
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    pub id: String,
    pub name: String,
    /// Shipped with the app (false = added by the user).
    pub builtin: bool,
    /// Has its own release sounds.
    pub has_release: bool,
    #[serde(skip)]
    pub dir: PathBuf,
}

fn read_config(dir: &Path) -> Option<Value> {
    let text = std::fs::read_to_string(dir.join("config.json")).ok()?;
    serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()
}

fn definitions(cfg: &Value) -> Option<&serde_json::Map<String, Value>> {
    cfg.get("definitions").or_else(|| cfg.get("defs")).and_then(Value::as_object)
}

fn is_keyboard(cfg: &Value) -> bool {
    if let Some(t) = cfg.get("soundpack_type").and_then(Value::as_str) {
        return !t.eq_ignore_ascii_case("mouse");
    }
    match definitions(cfg) {
        Some(defs) => !defs.keys().all(|k| k.starts_with("Mouse") || k.starts_with("Button") || k.starts_with("Wheel")),
        None => cfg.get("defines").is_some(),
    }
}

/// Every keyboard pack directly inside `root` (one folder per pack).
pub fn scan(root: &Path, builtin: bool) -> Vec<PackInfo> {
    let Ok(dir) = std::fs::read_dir(root) else { return Vec::new() };
    let mut out = Vec::new();
    for entry in dir.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(cfg) = read_config(&path) else { continue };
        if !is_keyboard(&cfg) {
            continue;
        }
        let folder = entry.file_name().to_string_lossy().into_owned();
        let name = cfg.get("name").and_then(Value::as_str).unwrap_or(&folder).trim().to_string();
        let has_release = definitions(&cfg)
            .map(|d| d.values().any(|v| v.get("timing").and_then(Value::as_array).is_some_and(|t| t.len() == 2)))
            .unwrap_or(false);
        let id = if builtin { folder } else { format!("user:{folder}") };
        out.push(PackInfo { id, name, builtin, has_release, dir: path });
    }
    out.sort_by_key(|p| p.name.to_lowercase());
    out
}

pub fn decode(path: &Path) -> Result<Buffer, String> {
    use rodio::Source;
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let dec = rodio::Decoder::new(BufReader::new(file)).map_err(|e| format!("{}: {e}", path.display()))?;
    let channels = dec.channels().max(1) as usize;
    let rate = dec.sample_rate();
    let raw: Vec<f32> = dec.convert_samples::<f32>().collect();
    let mut samples = Vec::with_capacity(raw.len() / channels * 2);
    for frame in raw.chunks(channels) {
        let l = frame[0];
        let r = if channels > 1 { frame[1] } else { l };
        samples.push(l);
        samples.push(r);
    }
    Ok(Buffer { samples, rate })
}

fn segment(buf: &Arc<Buffer>, start_ms: f64, end_ms: f64) -> Option<Segment> {
    let to_frame = |ms: f64| ((ms.max(0.0) / 1000.0) * buf.rate as f64) as usize;
    let start = to_frame(start_ms).min(buf.frames());
    let end = to_frame(end_ms).min(buf.frames());
    (end > start + 8).then(|| Segment { buf: buf.clone(), start, end })
}

fn whole(buf: &Arc<Buffer>) -> Option<Segment> {
    (buf.frames() > 8).then(|| Segment { buf: buf.clone(), start: 0, end: buf.frames() })
}

/// Load a pack's audio. Files are decoded once and shared between keys.
pub fn load(dir: &Path) -> Result<Pack, String> {
    let cfg = read_config(dir).ok_or("config.json is missing or not valid JSON")?;
    let mut files: HashMap<String, Arc<Buffer>> = HashMap::new();
    let mut open = |name: &str| -> Result<Arc<Buffer>, String> {
        let name = name.trim_start_matches("./");
        if let Some(b) = files.get(name) {
            return Ok(b.clone());
        }
        let path = dir.join(name);
        if !path.starts_with(dir) || name.contains("..") {
            return Err(format!("{name}: outside the pack folder"));
        }
        let b = Arc::new(decode(&path)?);
        files.insert(name.to_string(), b.clone());
        Ok(b)
    };

    let mut keys = HashMap::new();
    if let Some(defs) = definitions(&cfg) {
        // V2
        let main = cfg.get("audio_file").and_then(Value::as_str);
        for (code, def) in defs {
            let file = def.get("audio_file").and_then(Value::as_str).or(main);
            let Some(file) = file else { continue };
            let buf = open(file)?;
            let timing: Vec<(f64, f64)> = def
                .get("timing")
                .and_then(Value::as_array)
                .map(|t| {
                    t.iter()
                        .filter_map(|p| Some((p.get(0)?.as_f64()?, p.get(1)?.as_f64()?)))
                        .collect()
                })
                .unwrap_or_default();
            let (down, up) = match timing.as_slice() {
                [] => (whole(&buf), None),
                [d] => (segment(&buf, d.0, d.1), None),
                [d, u] => (segment(&buf, d.0, d.1), segment(&buf, u.0, u.1)),
                _ => continue,
            };
            if let Some(down) = down {
                keys.insert(code.clone(), KeySound { down, up });
            }
        }
    } else if let Some(defines) = cfg.get("defines").and_then(Value::as_object) {
        // V1
        let multi = cfg.get("key_define_type").and_then(Value::as_str) == Some("multi");
        let main = cfg.get("sound").and_then(Value::as_str);
        for (raw, def) in defines {
            let Some(code) = raw.parse::<u32>().ok().and_then(keylock::keycode::from_uiohook) else { continue };
            let down = if multi {
                match def.as_str() {
                    Some(file) => whole(&open(file)?),
                    None => None,
                }
            } else {
                let (Some(file), Some(t)) = (main, def.as_array()) else { continue };
                let (Some(start), Some(len)) = (t.first().and_then(Value::as_f64), t.get(1).and_then(Value::as_f64)) else { continue };
                segment(&open(file)?, start, start + len)
            };
            if let Some(down) = down {
                keys.insert(code.to_string(), KeySound { down, up: None });
            }
        }
    } else {
        return Err("config.json has no key sounds (no `definitions` or `defines`)".into());
    }
    if keys.is_empty() {
        return Err("this pack has no playable key sounds".into());
    }
    let gain = cfg
        .pointer("/options/recommended_volume")
        .and_then(Value::as_f64)
        .unwrap_or(1.0)
        .clamp(0.1, 2.0) as f32;
    let mut names: Vec<String> = keys
        .keys()
        .filter(|k| k.starts_with("Key") || k.starts_with("Digit"))
        .cloned()
        .collect();
    if names.is_empty() {
        names = keys.keys().cloned().collect();
    }
    names.sort();
    Ok(Pack { keys, names, gain })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bundled_pack_loads() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/packs/keyboard");
        let packs = scan(&root, true);
        assert!(packs.len() >= 11, "found {} packs", packs.len());
        for info in packs {
            let pack = load(&info.dir).unwrap_or_else(|e| panic!("{}: {e}", info.id));
            for code in ["KeyA", "Space", "Enter", "Backspace", "ShiftLeft"] {
                let s = pack.sound_for(code).unwrap_or_else(|| panic!("{}: no {code}", info.id));
                let ms = (s.down.end - s.down.start) as f64 * 1000.0 / s.down.buf.rate as f64;
                assert!((10.0..2000.0).contains(&ms), "{} {code}: {ms} ms", info.id);
            }
            println!("{:28} {:2} keys, release: {}", info.id, pack.keys.len(), info.has_release);
        }
    }

    #[test]
    fn unknown_keys_get_a_stable_stand_in() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/packs/keyboard/eg-oreo");
        let pack = load(&root).unwrap();
        let a = pack.sound_for("Unidentified").unwrap().down.start;
        let b = pack.sound_for("Unidentified").unwrap().down.start;
        assert_eq!(a, b);
    }
}
