//! Keyboard sounds: a switch sound on every key press (and release), from Mechvibes packs.
//!
//! The `keylock` listener reports physical keys (never characters); each one plays its pack
//! sound, panned by where the key sits on a keyboard, with a little random pitch so repeated
//! presses don't sound machine-made. Preferences live in `sound.json` in the app data folder.
//! Built-in packs ship in `resources/packs/keyboard`; users add their own to `packs/` in the
//! app data folder (any Mechvibes pack folder works).

mod engine;
mod pack;

use engine::Engine;
use pack::{Pack, PackInfo, Segment};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

const DEFAULT_PACK: &str = "eg-oreo";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct SoundPrefs {
    pub enabled: bool,
    pub pack: String,
    /// 0 … 1.5 (1 is the pack's own level).
    pub volume: f32,
    /// Play a sound when a key comes back up too.
    pub release: bool,
    /// Keys on the left sound on the left.
    pub stereo: bool,
    /// A touch of random pitch on every press.
    pub vary_pitch: bool,
}

impl Default for SoundPrefs {
    fn default() -> Self {
        Self { enabled: true, pack: DEFAULT_PACK.into(), volume: 0.7, release: true, stereo: true, vary_pitch: true }
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SoundInfo {
    prefs: SoundPrefs,
    packs: Vec<PackInfo>,
    /// Key sounds work on this system.
    supported: bool,
    /// The key listener is running.
    listening: bool,
    error: Option<String>,
}

struct Inner {
    prefs: SoundPrefs,
    listener: Option<keylock::Listener>,
    loaded_pack: Option<String>,
    error: Option<String>,
}

/// Read from the key listener's thread, so kept outside the main lock.
struct Live {
    pack: Mutex<Option<Arc<Pack>>>,
    release: std::sync::atomic::AtomicBool,
    stereo: std::sync::atomic::AtomicBool,
    vary: std::sync::atomic::AtomicBool,
    seed: AtomicU32,
}

pub struct SoundState {
    inner: Mutex<Inner>,
    live: Arc<Live>,
    engine: Engine,
}

impl SoundState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner { prefs: SoundPrefs::default(), listener: None, loaded_pack: None, error: None }),
            live: Arc::new(Live {
                pack: Mutex::new(None),
                release: true.into(),
                stereo: true.into(),
                vary: true.into(),
                seed: AtomicU32::new(0x9E37_79B9),
            }),
            engine: Engine::start(),
        }
    }
}

// ---------- Where each key sits, for stereo ----------

/// Horizontal centre of a key on a standard keyboard's main block, in key widths (0 … 15).
/// Navigation keys and the numpad sit right of the main block.
fn key_x(code: &str) -> f32 {
    const ROWS: [(&str, f32, f32); 5] = [
        ("Backquote Digit1 Digit2 Digit3 Digit4 Digit5 Digit6 Digit7 Digit8 Digit9 Digit0 Minus Equal", 0.5, 1.0),
        ("KeyQ KeyW KeyE KeyR KeyT KeyY KeyU KeyI KeyO KeyP BracketLeft BracketRight", 2.0, 1.0),
        ("KeyA KeyS KeyD KeyF KeyG KeyH KeyJ KeyK KeyL Semicolon Quote", 2.25, 1.0),
        ("KeyZ KeyX KeyC KeyV KeyB KeyN KeyM Comma Period Slash", 2.75, 1.0),
        ("F1 F2 F3 F4", 2.5, 1.0),
    ];
    for (row, first, step) in ROWS {
        if let Some(i) = row.split(' ').position(|k| k == code) {
            return first + i as f32 * step;
        }
    }
    match code {
        "Escape" => 0.5,
        "F5" => 7.0,
        "F6" => 8.0,
        "F7" => 9.0,
        "F8" => 10.0,
        "F9" => 11.5,
        "F10" => 12.5,
        "F11" => 13.5,
        "F12" => 14.5,
        "Backspace" => 14.0,
        "Tab" => 0.75,
        "Backslash" => 14.25,
        "CapsLock" => 0.9,
        "Enter" => 13.9,
        "ShiftLeft" => 1.1,
        "IntlBackslash" => 1.8,
        "ShiftRight" => 13.6,
        "ControlLeft" => 0.6,
        "MetaLeft" => 1.9,
        "AltLeft" => 3.1,
        "Space" => 7.5,
        "AltRight" => 11.4,
        "MetaRight" => 12.6,
        "ContextMenu" => 13.4,
        "ControlRight" => 14.4,
        c if c.starts_with("Numpad") || c == "NumLock" => 17.5,
        "PrintScreen" | "ScrollLock" | "Pause" | "Insert" | "Home" | "PageUp" | "Delete" | "End" | "PageDown"
        | "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" => 15.8,
        _ => 7.5,
    }
}

/// -1 … 1. Kept short of the extremes: a real desk is wide, not hard-panned.
fn pan_for(code: &str) -> f32 {
    ((key_x(code) / 15.0) * 2.0 - 1.0).clamp(-1.0, 1.0) * 0.7
}

impl Live {
    /// 0 … 1, cheap and good enough for pitch jitter.
    fn random(&self) -> f32 {
        let mut x = self.seed.load(Ordering::Relaxed);
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.seed.store(x, Ordering::Relaxed);
        (x >> 8) as f32 / (1u32 << 24) as f32
    }

    fn on_key(&self, engine: &Engine, code: &str, down: bool) {
        let Some(pack) = self.pack.lock().ok().and_then(|p| p.clone()) else { return };
        let Some(sound) = pack.sound_for(code) else { return };
        let pan = if self.stereo.load(Ordering::Relaxed) { pan_for(code) } else { 0.0 };
        let jitter = if self.vary.load(Ordering::Relaxed) { 1.0 + (self.random() - 0.5) * 0.08 } else { 1.0 };
        if down {
            engine.play(&sound.down, pack.gain, pan, jitter);
        } else if self.release.load(Ordering::Relaxed) {
            match &sound.up {
                Some(up) => engine.play(up, pack.gain * 0.9, pan, jitter),
                None => {
                    // No release sound recorded: a softer, shorter, higher copy of the press.
                    let d = &sound.down;
                    let short = Segment { buf: d.buf.clone(), start: d.start, end: d.start + (d.end - d.start) * 3 / 5 };
                    engine.play(&short, pack.gain * 0.35, pan, jitter * 1.12);
                }
            }
        }
    }
}

// ---------- Packs on disk ----------

fn builtin_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .resolve("resources/packs/keyboard", tauri::path::BaseDirectory::Resource)
        .ok()
        .filter(|p| p.is_dir())
}

fn user_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join("packs"))
}

fn all_packs(app: &AppHandle) -> Vec<PackInfo> {
    let mut packs = builtin_dir(app).map(|d| pack::scan(&d, true)).unwrap_or_default();
    if let Some(d) = user_dir(app) {
        packs.extend(pack::scan(&d, false));
    }
    packs
}

fn prefs_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join("sound.json"))
}

fn save_prefs(app: &AppHandle, prefs: &SoundPrefs) {
    if let Some(path) = prefs_path(app) {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, serde_json::to_string_pretty(prefs).unwrap_or_default());
    }
}

// ---------- Applying preferences ----------

fn apply(app: &AppHandle, state: &SoundState, prefs: SoundPrefs) {
    let mut inner = state.inner.lock().unwrap();
    state.engine.set_volume(prefs.volume.clamp(0.0, 1.5));
    state.live.release.store(prefs.release, Ordering::Relaxed);
    state.live.stereo.store(prefs.stereo, Ordering::Relaxed);
    state.live.vary.store(prefs.vary_pitch, Ordering::Relaxed);
    inner.error = None;

    if prefs.enabled {
        if inner.loaded_pack.as_deref() != Some(prefs.pack.as_str()) {
            let packs = all_packs(app);
            let info = packs
                .iter()
                .find(|p| p.id == prefs.pack)
                .or_else(|| packs.iter().find(|p| p.id == DEFAULT_PACK))
                .or_else(|| packs.first());
            match info.map(|i| (i.id.clone(), pack::load(&i.dir))) {
                Some((id, Ok(p))) => {
                    *state.live.pack.lock().unwrap() = Some(Arc::new(p));
                    inner.loaded_pack = Some(id);
                }
                Some((id, Err(e))) => inner.error = Some(format!("Couldn't load the {id} sounds: {e}")),
                None => inner.error = Some("No sound packs were found.".into()),
            }
        }
        if inner.listener.is_none() && inner.error.is_none() {
            let live = state.live.clone();
            let engine = state.engine.clone();
            match keylock::listen(move |k| live.on_key(&engine, k.code, k.down)) {
                Ok(l) => inner.listener = Some(l),
                Err(e) => inner.error = Some(e.to_string()),
            }
        }
    } else if let Some(l) = inner.listener.take() {
        l.stop();
    }
    inner.prefs = prefs;
}

fn info(app: &AppHandle, state: &SoundState) -> SoundInfo {
    let packs = all_packs(app);
    let inner = state.inner.lock().unwrap();
    let mut prefs = inner.prefs.clone();
    if let Some(id) = &inner.loaded_pack {
        prefs.pack = id.clone(); // what's actually playing, if the saved one went missing
    }
    SoundInfo {
        prefs,
        packs,
        supported: keylock::CAN_LISTEN,
        listening: inner.listener.is_some(),
        error: inner.error.clone(),
    }
}

/// Load saved preferences and start listening. Call once at startup.
pub fn init(app: &AppHandle) {
    let prefs = prefs_path(app)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<SoundPrefs>(&t).ok())
        .unwrap_or_default();
    if let Some(d) = user_dir(app) {
        let _ = std::fs::create_dir_all(d);
    }
    apply(app, &app.state::<SoundState>(), prefs);
}

#[tauri::command]
pub fn get_sound(app: AppHandle, state: State<SoundState>) -> SoundInfo {
    info(&app, &state)
}

#[tauri::command]
pub async fn set_sound(app: AppHandle, prefs: SoundPrefs) -> SoundInfo {
    let state = app.state::<SoundState>();
    save_prefs(&app, &prefs);
    apply(&app, &state, prefs);
    info(&app, &state)
}

/// Type a short phrase with the current pack, so a pack can be heard before typing.
#[tauri::command]
pub fn sound_demo(app: AppHandle) {
    let state = app.state::<SoundState>();
    let live = state.live.clone();
    let engine = state.engine.clone();
    std::thread::spawn(move || {
        const KEYS: [&str; 16] = [
            "KeyH", "KeyE", "KeyL", "KeyL", "KeyO", "Space", "KeyC", "KeyL", "KeyA", "KeyC", "KeyK", "Space", "KeyJ", "KeyK",
            "Semicolon", "Enter",
        ];
        for (i, k) in KEYS.iter().enumerate() {
            live.on_key(&engine, k, true);
            std::thread::sleep(Duration::from_millis(55 + (live.random() * 50.0) as u64));
            live.on_key(&engine, k, false);
            let gap = if *k == "Space" || i % 5 == 4 { 140 } else { 45 };
            std::thread::sleep(Duration::from_millis(gap + (live.random() * 40.0) as u64));
        }
    });
}

/// Open the folder where users drop their own packs.
#[tauri::command]
pub fn open_packs_folder(app: AppHandle) {
    let Some(dir) = user_dir(&app) else { return };
    let _ = std::fs::create_dir_all(&dir);
    #[cfg(windows)]
    let cmd = "explorer";
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(all(unix, not(target_os = "macos")))]
    let cmd = "xdg-open";
    let _ = std::process::Command::new(cmd).arg(dir).spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_keys_pan_left() {
        assert!(pan_for("KeyQ") < -0.3);
        assert!(pan_for("KeyP") > 0.2);
        assert!(pan_for("Space").abs() < 0.05);
    }
}
