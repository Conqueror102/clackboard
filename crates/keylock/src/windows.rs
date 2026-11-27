//! Windows: a low-level keyboard hook (WH_KEYBOARD_LL) that swallows every key.
//!
//! Ctrl + Alt + Del and Win + L are handled by Windows itself and always get through, which is
//! a useful safety net. Without elevation the hook can't block keys while an elevated window
//! (e.g. Task Manager) has focus.

use crate::{Error, Event, Keyboard, Kind, Modifiers, Options, Permissions, Sink};
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    HC_ACTION, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN,
};

static SINK: Mutex<Option<Sink>> = Mutex::new(None);
static MODS: Modifiers = Modifiers::new();

const VK_U: u32 = 0x55;

pub struct Lock {
    thread_id: u32,
    thread: Option<JoinHandle<()>>,
}

impl Lock {
    pub fn release(&mut self) {
        if let Some(t) = self.thread.take() {
            unsafe {
                let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
            if t.thread().id() != std::thread::current().id() {
                let _ = t.join();
            }
        }
        *SINK.lock().unwrap() = None;
    }
}

pub fn keyboards() -> Vec<Keyboard> {
    vec![Keyboard { id: "all".into(), name: "All keyboards".into(), kind: Kind::All }]
}

pub fn permissions() -> Permissions {
    Permissions::default()
}

pub fn request_permissions() {}

fn label(vk: u32) -> String {
    match vk {
        0x41..=0x5A | 0x30..=0x39 => char::from_u32(vk).map(String::from).unwrap_or_default(),
        0x20 => "space".into(),
        0x0D => "⏎".into(),
        0x08 => "⌫".into(),
        0x1B => "Esc".into(),
        0x09 => "Tab".into(),
        _ => "·".into(),
    }
}

unsafe extern "system" fn hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code != HC_ACTION as i32 {
        return CallNextHookEx(None, code, wparam, lparam);
    }
    let info = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
    let msg = wparam.0 as u32;
    let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
    match info.vkCode {
        0x11 | 0xA2 | 0xA3 => MODS.ctrl.store(down, Ordering::Relaxed),
        0x12 | 0xA4 | 0xA5 => MODS.alt.store(down, Ordering::Relaxed),
        vk if down => {
            if let Some(sink) = SINK.lock().ok().and_then(|g| g.clone()) {
                if vk == VK_U && MODS.both() {
                    sink(Event::Escape);
                } else {
                    sink(Event::Swallowed { label: label(vk) });
                }
            }
        }
        _ => {}
    }
    LRESULT(1) // swallow every key, key-ups included
}

pub fn engage(_options: &Options, sink: Sink) -> Result<Lock, Error> {
    *SINK.lock().unwrap() = Some(sink);
    MODS.ctrl.store(false, Ordering::Relaxed);
    MODS.alt.store(false, Ordering::Relaxed);

    let (tx, rx) = mpsc::channel::<Result<u32, Error>>();
    let thread = std::thread::Builder::new()
        .name("keylock-hook".into())
        .spawn(move || unsafe {
            let module = match GetModuleHandleW(None) {
                Ok(m) => m,
                Err(e) => {
                    let _ = tx.send(Err(Error::Os(e.to_string())));
                    return;
                }
            };
            let hhook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook), HINSTANCE(module.0), 0) {
                Ok(h) => h,
                Err(e) => {
                    let _ = tx.send(Err(Error::Os(e.to_string())));
                    return;
                }
            };
            let _ = tx.send(Ok(GetCurrentThreadId()));
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {}
            let _ = UnhookWindowsHookEx(hhook);
        })
        .map_err(|e| Error::Os(e.to_string()))?;

    match rx.recv_timeout(Duration::from_secs(3)) {
        Ok(Ok(thread_id)) => Ok(Lock { thread_id, thread: Some(thread) }),
        Ok(Err(e)) => {
            let _ = thread.join();
            *SINK.lock().unwrap() = None;
            Err(e)
        }
        Err(_) => Err(Error::Os("the keyboard hook did not start in time".into())),
    }
}

// ---------- Listening: a second hook that lets every key through ----------
//
// One pass-through hook serves every listener (sounds, cleaning detection): it starts with the
// first and stops with the last.

static LISTENERS: Mutex<Vec<(u64, crate::KeySink)>> = Mutex::new(Vec::new());
static NEXT_LISTENER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
/// The hook thread: its id (for WM_QUIT) and handle.
static LISTEN_HOOK: Mutex<Option<(u32, JoinHandle<()>)>> = Mutex::new(None);
static HELD: crate::Held = crate::Held::new();

const LLKHF_EXTENDED: u32 = 0x01;
const VK_PAUSE: u32 = 0x13;
const VK_NUMLOCK: u32 = 0x90;

fn code_of(info: &KBDLLHOOKSTRUCT) -> (&'static str, usize) {
    let extended = info.flags.0 & LLKHF_EXTENDED != 0;
    let scan = info.scanCode as u16;
    let id = scan as usize | if extended { 0x100 } else { 0 };
    // Windows swaps the E0 flag on these two compared with the scan-code tables.
    let code = match info.vkCode {
        VK_PAUSE => Some("Pause"),
        VK_NUMLOCK => Some("NumLock"),
        _ => crate::keycode::from_set1(scan, extended),
    };
    (code.unwrap_or("Unidentified"), id)
}

unsafe extern "system" fn listen_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let info = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let msg = wparam.0 as u32;
        let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let (name, id) = code_of(info);
        if HELD.set(id, down) {
            let sinks: Vec<crate::KeySink> =
                LISTENERS.lock().map(|l| l.iter().map(|(_, s)| s.clone()).collect()).unwrap_or_default();
            for sink in sinks {
                sink(crate::KeyEvent { code: name, down });
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam) // never swallow
}

fn start_hook() -> Result<(u32, JoinHandle<()>), Error> {
    let (tx, rx) = mpsc::channel::<Result<u32, Error>>();
    let thread = std::thread::Builder::new()
        .name("keylock-listen".into())
        .spawn(move || unsafe {
            let module = match GetModuleHandleW(None) {
                Ok(m) => m,
                Err(e) => {
                    let _ = tx.send(Err(Error::Os(e.to_string())));
                    return;
                }
            };
            let hhook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(listen_hook), HINSTANCE(module.0), 0) {
                Ok(h) => h,
                Err(e) => {
                    let _ = tx.send(Err(Error::Os(e.to_string())));
                    return;
                }
            };
            let _ = tx.send(Ok(GetCurrentThreadId()));
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {}
            let _ = UnhookWindowsHookEx(hhook);
        })
        .map_err(|e| Error::Os(e.to_string()))?;
    match rx.recv_timeout(Duration::from_secs(3)) {
        Ok(Ok(thread_id)) => Ok((thread_id, thread)),
        Ok(Err(e)) => {
            let _ = thread.join();
            Err(e)
        }
        Err(_) => Err(Error::Os("the keyboard hook did not start in time".into())),
    }
}

pub struct Listener {
    id: u64,
}

impl Listener {
    pub fn stop(&mut self) {
        if self.id == 0 {
            return;
        }
        let id = std::mem::take(&mut self.id);
        let mut hook = LISTEN_HOOK.lock().unwrap();
        let empty = {
            let mut l = LISTENERS.lock().unwrap();
            l.retain(|(i, _)| *i != id);
            l.is_empty()
        };
        if empty {
            if let Some((thread_id, t)) = hook.take() {
                unsafe {
                    let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
                }
                if t.thread().id() != std::thread::current().id() {
                    let _ = t.join();
                }
                HELD.clear();
            }
        }
    }
}

pub fn listen(sink: crate::KeySink) -> Result<Listener, Error> {
    let mut hook = LISTEN_HOOK.lock().unwrap();
    if hook.is_none() {
        HELD.clear();
        *hook = Some(start_hook()?);
    }
    let id = NEXT_LISTENER.fetch_add(1, Ordering::Relaxed);
    LISTENERS.lock().unwrap().push((id, sink));
    Ok(Listener { id })
}
