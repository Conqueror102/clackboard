//! Turns Mechvibes V2 packs into small files for the website's typing demo.
//!
//! Browsers can't all play Ogg Vorbis (Safari), and a whole pack recording is several MB. This
//! keeps only the clips the keys use, mixes them to mono, resamples to 32 kHz, and writes one
//! 16-bit WAV per pack plus a JSON map of key → [press, release] times in the new file.
//!
//! Run from the repo root:
//!   cargo run -p site-packs --release -- src-tauri/resources/packs/keyboard site/packs eg-oreo …

use rodio::Source;
use serde_json::{json, Map, Value};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

const RATE: u32 = 32_000;
/// Silence between clips, so a clip played a hair long never runs into the next one.
const GAP_MS: f64 = 20.0;

fn decode(path: &Path) -> (Vec<f32>, u32) {
    let file = File::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let dec = rodio::Decoder::new(BufReader::new(file)).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let channels = dec.channels().max(1) as usize;
    let rate = dec.sample_rate();
    let raw: Vec<f32> = dec.convert_samples::<f32>().collect();
    let mono = raw.chunks(channels).map(|f| f.iter().sum::<f32>() / f.len() as f32).collect();
    (mono, rate)
}

/// The slice [start_ms, end_ms) of `src`, resampled to RATE.
fn slice(src: &[f32], src_rate: u32, start_ms: f64, end_ms: f64) -> Vec<f32> {
    let n = ((end_ms - start_ms).max(0.0) / 1000.0 * RATE as f64) as usize;
    let step = src_rate as f64 / RATE as f64;
    let first = start_ms / 1000.0 * src_rate as f64;
    (0..n)
        .map(|i| {
            let pos = first + i as f64 * step;
            let j = pos as usize;
            let t = (pos - j as f64) as f32;
            let a = src.get(j).copied().unwrap_or(0.0);
            let b = src.get(j + 1).copied().unwrap_or(a);
            a + (b - a) * t
        })
        .collect()
}

fn write_wav(path: &Path, samples: &[f32]) {
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    let data_len = (samples.len() * 2) as u32;
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    File::create(path).and_then(|mut f| f.write_all(&out)).unwrap();
}

fn convert(src_root: &Path, out_root: &Path, id: &str) {
    let dir = src_root.join(id);
    let cfg: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("config.json")).unwrap()).unwrap();
    let audio = cfg["audio_file"].as_str().expect("single-file V2 pack");
    let (src, src_rate) = decode(&dir.join(audio));
    let defs = cfg.get("definitions").or_else(|| cfg.get("defs")).and_then(Value::as_object).unwrap();

    let mut out: Vec<f32> = Vec::new();
    let mut keys = Map::new();
    let gap = vec![0.0; (GAP_MS / 1000.0 * RATE as f64) as usize];
    // Packs often share one recording between many keys: store each clip once.
    let mut seen: std::collections::HashMap<(u64, u64), (f64, f64)> = std::collections::HashMap::new();
    for (code, def) in defs {
        let mut segs = Vec::new();
        for pair in def["timing"].as_array().into_iter().flatten() {
            let (Some(a), Some(b)) = (pair[0].as_f64(), pair[1].as_f64()) else { continue };
            let (start, end) = *seen.entry((a.to_bits(), b.to_bits())).or_insert_with(|| {
                let clip = slice(&src, src_rate, a, b);
                let start = out.len() as f64 * 1000.0 / RATE as f64;
                out.extend_from_slice(&clip);
                let end = out.len() as f64 * 1000.0 / RATE as f64;
                out.extend_from_slice(&gap);
                (start, end)
            });
            segs.push(json!([(start * 10.0).round() / 10.0, (end * 10.0).round() / 10.0]));
        }
        if !segs.is_empty() {
            keys.insert(code.clone(), Value::Array(segs));
        }
    }
    std::fs::create_dir_all(out_root).unwrap();
    write_wav(&out_root.join(format!("{id}.wav")), &out);
    let meta = json!({ "id": id, "name": cfg["name"], "author": cfg["author"], "keys": keys });
    std::fs::write(out_root.join(format!("{id}.json")), serde_json::to_string(&meta).unwrap()).unwrap();
    println!("{id}: {} keys, {:.1} s, {} KB", keys.len(), out.len() as f64 / RATE as f64, out.len() * 2 / 1024);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [src, out, ids @ ..] = args.as_slice() else {
        eprintln!("usage: site-packs <packs/keyboard> <out dir> <pack id>…");
        std::process::exit(2);
    };
    for id in ids {
        convert(Path::new(src), Path::new(out), id);
    }
}
