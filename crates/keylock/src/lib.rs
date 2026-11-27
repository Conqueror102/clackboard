//! Temporarily swallow keyboard input so a keyboard can be cleaned.
//!
//! * macOS: preferably a hardware lock: a root helper (installed once with the admin password,
//!   then allowed in Input Monitoring) seizes the keyboards through IOKit. Without the helper, a
//!   HID-level `CGEventTap` that deletes key events (needs Accessibility and Input Monitoring;
//!   blind while another process holds Secure Input).
//! * Windows: a `WH_KEYBOARD_LL` hook that returns 1 for every key (no permission needed).
//! * Linux: an exclusive `EVIOCGRAB` on each selected `/dev/input/event*` keyboard (needs read
//!   access to the device, e.g. membership of the `input` group). Works on X11 and Wayland.
//!
//! On every platform the escape shortcut **Ctrl + Alt + U** is still detected while locked and
//! reported as [`Event::Escape`]. The pointer is never touched. Nothing typed is stored.

use std::fmt;
use std::sync::Arc;

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod detect;
pub mod keycode;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod mac_helper;
#[cfg(target_os = "macos")]
mod mac_hid;
#[cfg(target_os = "macos")]
mod macos;
// Named `win` so it doesn't clash with the `windows` crate.
#[cfg(windows)]
#[path = "windows.rs"]
mod win;

#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(windows)]
use win as platform;

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
mod platform {
    use super::*;
    pub struct Lock;
    pub fn keyboards() -> Vec<Keyboard> {
        Vec::new()
    }
    pub fn permissions() -> Permissions {
        Permissions::default()
    }
    pub fn request_permissions() {}
    pub fn engage(_: &Options, _: Sink) -> Result<Lock, Error> {
        Err(Error::Unsupported("this operating system is not supported".into()))
    }
    impl Lock {
        pub fn release(&mut self) {}
    }
}

/// Where a keyboard is connected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    BuiltIn,
    Usb,
    Bluetooth,
    Other,
    /// The platform locks every keyboard at once (macOS, Windows).
    All,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::BuiltIn => "built-in",
            Kind::Usb => "usb",
            Kind::Bluetooth => "bluetooth",
            Kind::Other => "other",
            Kind::All => "all",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Keyboard {
    /// Stable id to pass back in [`Options::device_ids`].
    pub id: String,
    pub name: String,
    pub kind: Kind,
}

/// What the lock reports while it is engaged.
#[derive(Debug, Clone)]
pub enum Event {
    /// A key press was dropped. `label` is a short legend ("A", "space", "⏎") for the on-screen
    /// keycap, or "·" when there is no simple legend. It is never logged or stored.
    Swallowed { label: String },
    /// The escape shortcut (Ctrl + Alt + U) was pressed.
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    Granted,
    Denied,
    /// This permission does not exist on this platform.
    #[default]
    NotNeeded,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Granted => "granted",
            Status::Denied => "denied",
            Status::NotNeeded => "not-needed",
        }
    }
}

/// The macOS keyboard helper (hardware lock).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Helper {
    /// Not part of this platform.
    #[default]
    NotNeeded,
    Missing,
    /// Installed, but not yet switched on in Input Monitoring.
    NeedsApproval,
    /// Installed from an older version of the app.
    Outdated,
    Ready,
}

impl Helper {
    pub fn as_str(self) -> &'static str {
        match self {
            Helper::NotNeeded => "not-needed",
            Helper::Missing => "missing",
            Helper::NeedsApproval => "needs-approval",
            Helper::Outdated => "outdated",
            Helper::Ready => "ready",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Permissions {
    /// macOS: System Settings → Privacy & Security → Accessibility.
    pub accessibility: Status,
    /// macOS: System Settings → Privacy & Security → Input Monitoring.
    pub input_monitoring: Status,
    /// Linux: read access to `/dev/input/event*`.
    pub input_devices: Status,
    /// macOS: the keyboard helper. When it is ready, the other macOS permissions aren't needed.
    pub helper: Helper,
}

impl Permissions {
    /// Everything the lock needs is in place. With the macOS helper installed, that means the
    /// helper is ready (an installed helper is never silently bypassed); without it, the
    /// fallback's own permissions.
    pub fn all_granted(&self) -> bool {
        match self.helper {
            Helper::Ready => true,
            Helper::NeedsApproval | Helper::Outdated => false,
            Helper::NotNeeded | Helper::Missing => [self.accessibility, self.input_monitoring, self.input_devices]
                .iter()
                .all(|s| *s != Status::Denied),
        }
    }
}

/// Everything the UI shows about the system, read together.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub permissions: Permissions,
    pub keyboards: Vec<Keyboard>,
    /// macOS: the app holding Secure Input, which blinds the no-helper fallback.
    pub secure_input_holder: Option<String>,
}

/// Read permissions, keyboards and Secure Input in one go. `thorough` (macOS) also verifies the
/// helper's permission from the user's side, which catches a permission switched off since the
/// helper started; it launches a short-lived process, so call it every few seconds, not more.
pub fn snapshot(thorough: bool) -> Snapshot {
    #[cfg(target_os = "macos")]
    {
        let (permissions, keyboards) = macos::snapshot(thorough);
        let secure_input_holder =
            if permissions.helper == Helper::Ready { None } else { macos::secure_input_holder() };
        Snapshot { permissions, keyboards, secure_input_holder }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = thorough;
        Snapshot { permissions: platform::permissions(), keyboards: platform::keyboards(), secure_input_holder: None }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Keyboards to lock (ids from [`keyboards`]). Empty means every keyboard.
    /// Linux, and macOS with the keyboard helper, lock one keyboard at a time; otherwise every
    /// keyboard is locked.
    pub device_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Error {
    PermissionDenied(String),
    NoKeyboards,
    Unsupported(String),
    Os(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::PermissionDenied(m) => write!(f, "permission denied: {m}"),
            Error::NoKeyboards => write!(f, "no keyboards found to lock"),
            Error::Unsupported(m) => write!(f, "unsupported: {m}"),
            Error::Os(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for Error {}

pub(crate) type Sink = Arc<dyn Fn(Event) + Send + Sync + 'static>;
#[allow(dead_code)]
pub(crate) type CleaningSink = Arc<dyn Fn() + Send + Sync + 'static>;

/// An engaged keyboard lock. The keyboard is released when this is dropped.
pub struct Lock {
    inner: platform::Lock,
}

impl Lock {
    /// Release the keyboard now. Dropping the lock does the same.
    pub fn release(mut self) {
        self.inner.release();
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        self.inner.release();
    }
}

/// Watches for a cloth wiping across the keys (many keys at once) and reports it, so a lock can
/// start by itself. macOS uses its own event tap or the keyboard helper; Windows and Linux use
/// the pass-through key listener.
pub struct Monitor {
    #[cfg(target_os = "macos")]
    inner: macos::Monitor,
    #[cfg(any(windows, target_os = "linux"))]
    _listener: Listener,
}

#[cfg(target_os = "macos")]
impl Drop for Monitor {
    fn drop(&mut self) {
        self.inner.stop();
    }
}

impl Monitor {
    pub fn alive(&self) -> bool {
        #[cfg(target_os = "macos")]
        return self.inner.alive();
        #[cfg(any(windows, target_os = "linux"))]
        return true;
        #[cfg(not(any(target_os = "macos", windows, target_os = "linux")))]
        false
    }

    pub fn via_helper(&self) -> bool {
        #[cfg(target_os = "macos")]
        return self.inner.via_helper();
        #[cfg(not(target_os = "macos"))]
        false
    }
}

pub const CAN_MONITOR: bool = cfg!(any(target_os = "macos", windows, target_os = "linux"));

/// A small stable number for a key name, for the detector.
#[allow(dead_code)]
fn key_id(code: &str) -> u32 {
    code.bytes().fold(2166136261u32, |h, b| (h ^ b as u32).wrapping_mul(16777619))
}

pub fn monitor<F>(on_cleaning: F) -> Result<Monitor, Error>
where
    F: Fn() + Send + Sync + 'static,
{
    #[cfg(target_os = "macos")]
    {
        macos::monitor(Arc::new(on_cleaning)).map(|inner| Monitor { inner })
    }
    #[cfg(any(windows, target_os = "linux"))]
    {
        use detect::{Detector, Edge};
        let detector = std::sync::Mutex::new(Detector::default());
        let listener = listen(move |k: KeyEvent| {
            // Shortcuts and games hold several modifiers at once; a cloth hits the letters.
            if matches!(
                k.code,
                "ShiftLeft" | "ShiftRight" | "ControlLeft" | "ControlRight" | "AltLeft" | "AltRight" | "MetaLeft"
                    | "MetaRight" | "CapsLock" | "Fn"
            ) {
                return;
            }
            let id = key_id(k.code);
            let edge = if k.down { Edge::Down(id) } else { Edge::Up(id) };
            let fired = detector.lock().map(|mut d| d.push(std::time::Instant::now(), edge)).unwrap_or(false);
            if fired {
                on_cleaning();
            }
        })?;
        Ok(Monitor { _listener: listener })
    }
    #[cfg(not(any(target_os = "macos", windows, target_os = "linux")))]
    {
        let _ = on_cleaning;
        Err(Error::Unsupported("detecting cleaning is not available on this system yet".into()))
    }
}

pub fn request_listen_access() {
    #[cfg(target_os = "macos")]
    macos::request_listen_access();
}

/// Keyboards this platform can lock. macOS and Windows return one "All keyboards" entry.
pub fn keyboards() -> Vec<Keyboard> {
    platform::keyboards()
}

pub fn permissions() -> Permissions {
    platform::permissions()
}

/// Ask the OS for the permissions the lock needs (macOS shows its prompts; elsewhere a no-op).
pub fn request_permissions() {
    platform::request_permissions()
}

/// macOS: install the keyboard helper. Shows the system admin-password prompt and blocks until
/// the user answers. Elsewhere: unsupported.
pub fn install_helper() -> Result<(), Error> {
    #[cfg(target_os = "macos")]
    return macos::install_helper();
    #[cfg(not(target_os = "macos"))]
    Err(Error::Unsupported("the keyboard helper is macOS only".into()))
}

/// macOS: remove the keyboard helper (asks for the admin password).
pub fn uninstall_helper() -> Result<(), Error> {
    #[cfg(target_os = "macos")]
    return macos::uninstall_helper();
    #[cfg(not(target_os = "macos"))]
    Err(Error::Unsupported("the keyboard helper is macOS only".into()))
}

/// macOS: open System Settings at Input Monitoring, where the helper is switched on.
pub fn open_helper_settings() {
    #[cfg(target_os = "macos")]
    macos::open_helper_settings();
}

/// macOS: show the system's Input Monitoring prompt for the keyboard helper (also lists it in
/// System Settings). Install already does this once.
pub fn request_helper_access() {
    #[cfg(target_os = "macos")]
    macos::request_helper_access();
}

/// Call first thing in `main`. When the executable was started as the keyboard helper
/// (`--keyhelper`, `--keyhelper-install`, `--keyhelper-uninstall`), runs that and returns the
/// exit code; otherwise returns `None` and the app starts normally.
pub fn helper_main() -> Option<i32> {
    #[cfg(target_os = "macos")]
    return mac_helper::main_from_args();
    #[cfg(not(target_os = "macos"))]
    None
}

/// Start swallowing key presses. `on_event` runs on the lock's own thread: keep it short, and
/// never drop the returned [`Lock`] from inside it (hand off to another thread instead).
pub fn engage<F>(options: &Options, on_event: F) -> Result<Lock, Error>
where
    F: Fn(Event) + Send + Sync + 'static,
{
    let inner = platform::engage(options, Arc::new(on_event))?;
    Ok(Lock { inner })
}

// ---------- Listening (keys pass through untouched) ----------

/// A physical key went down or came up. Auto-repeat is filtered out: a held key gives one
/// `down`, then one `up`. Only the key's position is reported, never the character it types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    /// W3C `KeyboardEvent.code` name ("KeyA", "Space"), or "Unidentified".
    pub code: &'static str,
    pub down: bool,
}

pub(crate) type KeySink = Arc<dyn Fn(KeyEvent) + Send + Sync + 'static>;

/// This platform can listen to keys.
pub const CAN_LISTEN: bool = cfg!(any(windows, target_os = "linux", target_os = "macos"));

/// A running key listener. Stops when dropped.
pub struct Listener {
    #[cfg(any(windows, target_os = "linux", target_os = "macos"))]
    inner: platform::Listener,
}

impl Listener {
    pub fn stop(self) {}
}

#[cfg(any(windows, target_os = "linux", target_os = "macos"))]
impl Drop for Listener {
    fn drop(&mut self) {
        self.inner.stop();
    }
}

/// Hear every key press and release without blocking it, for sounds. `on_key` runs on the
/// listener's own thread (on Windows, inside the keyboard hook): keep it very short.
///
/// * Windows: a pass-through `WH_KEYBOARD_LL` hook. No permission needed.
/// * Linux: reads `/dev/input/event*` keyboards without grabbing them (the `input` group).
/// * macOS: a listen-only session event tap (Input Monitoring). Silent during Secure Input.
pub fn listen<F>(on_key: F) -> Result<Listener, Error>
where
    F: Fn(KeyEvent) + Send + Sync + 'static,
{
    #[cfg(any(windows, target_os = "linux", target_os = "macos"))]
    {
        platform::listen(Arc::new(on_key)).map(|inner| Listener { inner })
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = on_key;
        Err(Error::Unsupported("keyboard sounds are not available on this system yet".into()))
    }
}

/// Remembers which keys are held, so auto-repeat can be dropped. Indexed by a small key id.
#[allow(dead_code)]
pub(crate) struct Held([std::sync::atomic::AtomicU64; 8]);

#[allow(dead_code)]
impl Held {
    pub const fn new() -> Self {
        use std::sync::atomic::AtomicU64;
        Self([const { AtomicU64::new(0) }; 8])
    }
    /// Record a transition; returns false when nothing changed (a repeat, or a stray up).
    pub fn set(&self, id: usize, down: bool) -> bool {
        use std::sync::atomic::Ordering::Relaxed;
        let id = id % 512;
        let bit = 1u64 << (id % 64);
        let word = &self.0[id / 64];
        let was = if down { word.fetch_or(bit, Relaxed) } else { word.fetch_and(!bit, Relaxed) };
        (was & bit != 0) != down
    }
    pub fn clear(&self) {
        for w in &self.0 {
            w.store(0, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// Tracks Ctrl and Alt so the escape shortcut can be recognised from raw key events.
#[allow(dead_code)]
#[derive(Default)]
pub(crate) struct Modifiers {
    pub ctrl: std::sync::atomic::AtomicBool,
    pub alt: std::sync::atomic::AtomicBool,
}

#[allow(dead_code)]
impl Modifiers {
    pub const fn new() -> Self {
        Self {
            ctrl: std::sync::atomic::AtomicBool::new(false),
            alt: std::sync::atomic::AtomicBool::new(false),
        }
    }
    pub fn both(&self) -> bool {
        use std::sync::atomic::Ordering::Relaxed;
        self.ctrl.load(Relaxed) && self.alt.load(Relaxed)
    }
}
