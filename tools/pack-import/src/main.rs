//! Imports other keyboard sound sets as Clackboard packs (Mechvibes V2, one WAV per pack).
//!
//! * `ks <profile dir> <out dir> <name>`: a Keyboard Sounds profile (`profile.yaml` plus audio).
//!   Its recordings are shared out over the keys: each key always plays the same recording, as
//!   on a real board, and keys the profile maps (space, enter…) get their own.
//! * `bucklespring <wav dir> <out dir> <name>`: bucklespring's per-key recordings
//!   (`<evdev code in hex>-0.wav` press, `-1.wav` release).
//!
//! Clips are mixed to mono at 44.1 kHz with leading and trailing silence trimmed, so a press
//! sounds the moment the key goes down.

use rodio::Source;
use serde_json::{json, Map, Value};
use serde_yaml::Value as Yaml;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

const RATE: u32 = 44_100;
const GAP: usize = (RATE / 50) as usize; // 20 ms of silence between clips

fn decode(path: &Path) -> Vec<f32> {
    let file = File::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let dec = rodio::Decoder::new(BufReader::new(file)).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let channels = dec.channels().max(1) as usize;
    let rate = dec.sample_rate();
    let raw: Vec<f32> = dec.convert_samples::<f32>().collect();
    let mono: Vec<f32> = raw.chunks(channels).map(|f| f.iter().sum::<f32>() / f.len() as f32).collect();
    let step = rate as f64 / RATE as f64;
    let n = (mono.len() as f64 / step) as usize;
    let resampled = (0..n)
        .map(|i| {
            let pos = i as f64 * step;
            let j = pos as usize;
            let t = (pos - j as f64) as f32;
            let a = mono.get(j).copied().unwrap_or(0.0);
            let b = mono.get(j + 1).copied().unwrap_or(a);
            a + (b - a) * t
        })
        .collect();
    trim(resampled)
}

/// Drop silence before the attack (keeping 2 ms) and after the tail (keeping 15 ms).
fn trim(s: Vec<f32>) -> Vec<f32> {
    let peak = s.iter().fold(0f32, |m, x| m.max(x.abs()));
    if peak <= 0.0 {
        return s;
    }
    let start_at = peak * 0.04;
    let end_at = peak * 0.004;
    let first = s.iter().position(|x| x.abs() > start_at).unwrap_or(0);
    let last = s.iter().rposition(|x| x.abs() > end_at).unwrap_or(s.len() - 1);
    let a = first.saturating_sub(RATE as usize / 500);
    let b = (last + RATE as usize * 15 / 1000).min(s.len());
    s[a..b].to_vec()
}

/// Every key Clackboard knows, as W3C names.
fn all_codes() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = (1..=0x58u16).filter_map(|s| keylock::keycode::from_set1(s, false)).collect();
    for s in [0x1C, 0x1D, 0x35, 0x37, 0x38, 0x47, 0x48, 0x49, 0x4B, 0x4D, 0x4F, 0x50, 0x51, 0x52, 0x53, 0x5B, 0x5C, 0x5D] {
        if let Some(c) = keylock::keycode::from_set1(s, true) {
            v.push(c);
        }
    }
    v.push("Pause");
    v.sort_unstable();
    v.dedup();
    v
}

fn stable_hash(s: &str) -> usize {
    s.bytes().fold(2166136261u32, |h, b| (h ^ b as u32).wrapping_mul(16777619)) as usize
}

/// Keyboard Sounds key names (pynput style) to W3C names.
fn ks_codes(name: &str) -> Vec<&'static str> {
    let n = name.to_lowercase();
    let v: &[&'static str] = match n.as_str() {
        "space" => &["Space"],
        "enter" | "return" => &["Enter", "NumpadEnter"],
        "backspace" => &["Backspace"],
        "delete" => &["Delete"],
        "tab" => &["Tab"],
        "shift" => &["ShiftLeft", "ShiftRight"],
        "shift_l" => &["ShiftLeft"],
        "shift_r" => &["ShiftRight"],
        "caps_lock" => &["CapsLock"],
        "ctrl" => &["ControlLeft", "ControlRight"],
        "ctrl_l" => &["ControlLeft"],
        "ctrl_r" => &["ControlRight"],
        "alt" => &["AltLeft", "AltRight"],
        "alt_l" => &["AltLeft"],
        "alt_r" | "alt_gr" => &["AltRight"],
        "cmd" => &["MetaLeft", "MetaRight"],
        "cmd_l" => &["MetaLeft"],
        "cmd_r" => &["MetaRight"],
        "esc" | "escape" => &["Escape"],
        "up" => &["ArrowUp"],
        "down" => &["ArrowDown"],
        "left" => &["ArrowLeft"],
        "right" => &["ArrowRight"],
        "home" => &["Home"],
        "end" => &["End"],
        "page_up" => &["PageUp"],
        "page_down" => &["PageDown"],
        "insert" => &["Insert"],
        _ => &[],
    };
    v.to_vec()
}

struct Builder {
    out: Vec<f32>,
    /// file path -> [start_ms, end_ms]
    clips: HashMap<String, (f64, f64)>,
}

impl Builder {
    fn clip(&mut self, path: &Path) -> (f64, f64) {
        let key = path.to_string_lossy().into_owned();
        if let Some(c) = self.clips.get(&key) {
            return *c;
        }
        let s = decode(path);
        let ms = |n: usize| (n as f64 * 1000.0 / RATE as f64 * 10.0).round() / 10.0;
        let start = ms(self.out.len());
        self.out.extend_from_slice(&s);
        let end = ms(self.out.len());
        self.out.extend(std::iter::repeat_n(0.0, GAP));
        self.clips.insert(key, (start, end));
        (start, end)
    }
}

fn write_pack(out_dir: &Path, name: &str, author: &str, description: &str, b: &Builder, defs: BTreeMap<&str, Value>) {
    std::fs::create_dir_all(out_dir).unwrap();
    // 16-bit mono WAV.
    let mut w = Vec::with_capacity(44 + b.out.len() * 2);
    let len = (b.out.len() * 2) as u32;
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + len).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&RATE.to_le_bytes());
    w.extend_from_slice(&(RATE * 2).to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&len.to_le_bytes());
    for s in &b.out {
        w.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    File::create(out_dir.join("sound.wav")).and_then(|mut f| f.write_all(&w)).unwrap();

    let definitions: Map<String, Value> = defs.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    let cfg = json!({
        "config_version": "2",
        "id": out_dir.file_name().unwrap().to_string_lossy(),
        "name": name,
        "author": author,
        "description": description,
        "definition_method": "single",
        "audio_file": "sound.wav",
        "options": { "recommended_volume": 1.0, "random_pitch": false },
        "definitions": definitions,
    });
    std::fs::write(out_dir.join("config.json"), serde_json::to_string_pretty(&cfg).unwrap()).unwrap();
    println!(
        "{:28} {:3} keys, {:2} recordings, {:5} KB",
        out_dir.file_name().unwrap().to_string_lossy(),
        definitions.len(),
        b.clips.len(),
        w.len() / 1024
    );
}

fn ids(v: &Yaml) -> Vec<String> {
    match v {
        Yaml::String(s) => vec![s.clone()],
        Yaml::Sequence(seq) => seq.iter().filter_map(|x| x.as_str().map(String::from)).collect(),
        _ => Vec::new(),
    }
}

fn import_ks(dir: &Path, out: &Path, name: &str) {
    let y: Yaml = serde_yaml::from_str(&std::fs::read_to_string(dir.join("profile.yaml")).unwrap()).unwrap();
    let author = y["profile"]["author"].as_str().unwrap_or("Keyboard Sounds").to_string();
    let description = y["profile"]["description"].as_str().unwrap_or("").to_string();

    // id -> (press file, release file)
    let mut sources: Vec<(String, String, Option<String>)> = Vec::new();
    for s in y["sources"].as_sequence().expect("sources") {
        let id = match &s["id"] {
            Yaml::String(x) => x.clone(),
            other => serde_yaml::to_string(other).unwrap().trim().to_string(),
        };
        match &s["source"] {
            Yaml::String(f) => sources.push((id, f.clone(), None)),
            m => sources.push((
                id,
                m["press"].as_str().expect("press").to_string(),
                m["release"].as_str().map(String::from),
            )),
        }
    }
    let by_id: HashMap<&str, (&str, Option<&str>)> =
        sources.iter().map(|(i, p, r)| (i.as_str(), (p.as_str(), r.as_deref()))).collect();

    let keys = &y["keys"];
    let mut default = ids(&keys["default"]);
    if default.is_empty() {
        default = sources.iter().map(|s| s.0.clone()).collect();
    }
    let mut mapped: HashMap<&'static str, Vec<String>> = HashMap::new();
    for m in keys["other"].as_sequence().cloned().unwrap_or_default() {
        let sounds = ids(&m["sound"]);
        for k in m["keys"].as_sequence().cloned().unwrap_or_default() {
            for code in k.as_str().map(ks_codes).unwrap_or_default() {
                mapped.insert(code, sounds.clone());
            }
        }
    }

    let mut b = Builder { out: Vec::new(), clips: HashMap::new() };
    let mut defs = BTreeMap::new();
    for code in all_codes() {
        let choices = mapped.get(code).unwrap_or(&default);
        let id = &choices[stable_hash(code) % choices.len()];
        let Some((press, release)) = by_id.get(id.as_str()) else { continue };
        let p = b.clip(&dir.join(press));
        let mut timing = vec![json!([p.0, p.1])];
        if let Some(r) = release {
            let r = b.clip(&dir.join(r));
            timing.push(json!([r.0, r.1]));
        }
        defs.insert(code, json!({ "timing": timing }));
    }
    write_pack(out, name, &author, &description, &b, defs);
}

fn import_bucklespring(dir: &Path, out: &Path, name: &str) {
    let mut b = Builder { out: Vec::new(), clips: HashMap::new() };
    let mut defs = BTreeMap::new();
    let mut files: Vec<_> = std::fs::read_dir(dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    files.sort();
    for f in files.iter().filter(|f| f.ends_with("-0.wav")) {
        let hex = f.trim_end_matches("-0.wav");
        let Ok(code) = u16::from_str_radix(hex, 16) else { continue };
        let Some(w3c) = keylock::keycode::from_evdev(code) else { continue };
        let p = b.clip(&dir.join(f));
        let mut timing = vec![json!([p.0, p.1])];
        let rel = dir.join(format!("{hex}-1.wav"));
        if rel.exists() {
            let r = b.clip(&rel);
            timing.push(json!([r.0, r.1]));
        }
        defs.insert(w3c, json!({ "timing": timing }));
    }
    write_pack(
        out,
        name,
        "bucklespring (Ico Doornekamp)",
        "Every key of an IBM Model M recorded separately, press and release.",
        &b,
        defs,
    );
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    match a.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["ks", dir, out, name] => import_ks(Path::new(dir), Path::new(out), name),
        ["bucklespring", dir, out, name] => import_bucklespring(Path::new(dir), Path::new(out), name),
        _ => {
            eprintln!("usage: pack-import ks <profile dir> <out dir> <name>\n       pack-import bucklespring <wav dir> <out dir> <name>");
            std::process::exit(2);
        }
    }
}
