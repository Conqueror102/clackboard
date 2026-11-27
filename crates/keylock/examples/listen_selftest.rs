//! Windows self-test for the pass-through listener and cleaning detection.
//!
//! Sends synthetic key presses (five letters at once, like a cloth) and checks that:
//!   1. the listener hears every press and release, with the right key names;
//!   2. two listeners can run at once (sounds + cleaning detection share one hook);
//!   3. the cleaning monitor fires once.
//! A keyboard lock is engaged first, so the synthetic keys never reach any app. Keys you press
//! during the ~1 second test are swallowed too.
//!
//! Run: cargo run -p keylock --example listen_selftest

#[cfg(not(windows))]
fn main() {
    eprintln!("listen_selftest is Windows only");
}

#[cfg(windows)]
fn main() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
        VIRTUAL_KEY,
    };

    // Installed first, so it runs last: the listeners see each key, then the lock drops it.
    let lock = keylock::engage(&keylock::Options::default(), |_| {}).expect("lock");

    let heard: Arc<Mutex<Vec<(String, bool)>>> = Arc::default();
    let h = heard.clone();
    let listener = keylock::listen(move |k| h.lock().unwrap().push((k.code.to_string(), k.down))).expect("listen");
    let second = Arc::new(AtomicUsize::new(0));
    let s = second.clone();
    let listener2 = keylock::listen(move |_| {
        s.fetch_add(1, Ordering::SeqCst);
    })
    .expect("second listener");
    let fired = Arc::new(AtomicUsize::new(0));
    let f = fired.clone();
    let monitor = keylock::monitor(move || {
        f.fetch_add(1, Ordering::SeqCst);
    })
    .expect("monitor");

    let send = |scan: u16, up: bool| {
        let flags = KEYEVENTF_SCANCODE | if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) };
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT { wVk: VIRTUAL_KEY(0), wScan: scan, dwFlags: flags, time: 0, dwExtraInfo: 0 },
            },
        };
        unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
    };
    // A, S, D, F, G down together (a cloth), then a repeat of A (should be ignored), then up.
    let keys = [0x1E, 0x1F, 0x20, 0x21, 0x22];
    for k in keys {
        send(k, false);
        std::thread::sleep(Duration::from_millis(10));
    }
    send(0x1E, false); // auto-repeat
    for k in keys {
        send(k, true);
    }
    std::thread::sleep(Duration::from_millis(300));

    drop(monitor);
    drop(listener2);
    drop(listener);
    lock.release();

    let heard = heard.lock().unwrap().clone();
    let downs: Vec<&str> = heard.iter().filter(|(_, d)| *d).map(|(c, _)| c.as_str()).collect();
    let ups = heard.iter().filter(|(_, d)| !*d).count();
    println!("heard downs: {downs:?}, ups: {ups}");
    println!("second listener events: {}", second.load(Ordering::SeqCst));
    println!("cleaning detected: {} time(s)", fired.load(Ordering::SeqCst));
    let ok = downs == ["KeyA", "KeyS", "KeyD", "KeyF", "KeyG"]
        && ups == 5
        && second.load(Ordering::SeqCst) == 10
        && fired.load(Ordering::SeqCst) == 1;
    println!("{}", if ok { "PASS" } else { "FAIL" });
    std::process::exit(if ok { 0 } else { 1 });
}
