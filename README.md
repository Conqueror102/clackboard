# Clackboard

**Type like it's mechanical.** Clackboard plays a real mechanical switch sound for every key you press and release, on macOS, Windows and Linux. It can also lock your keyboard while you wipe it clean.

**[Website and live demo](https://conqueror102.github.io/clackboard/)** · Free and open source (MIT) · [Install guide](docs/installing.md) · [What's new](CHANGELOG.md) · [Credits](CREDITS.md)

## Sounds

- **A sound on every press and release.** Each key has its own recorded sound, so the space bar thocks and the letters click.
- **26 switches built in:** clicky (Cherry MX Blue, IBM Model M, a typewriter), tactile (Cherry MX Brown, Holy Panda, EG Oreo, EG Crystal Purple, Logitech G915), linear (Cherry MX Black, Red and Speed Silver, Gateron Ink, NovelKeys Cream, Telios V2, Banana Split, Alpaca, SteelSeries Apex Pro) and Topre. Press **Play a typing demo** to hear one.
- **Like a real desk:** keys on the left sound on the left, keys on the right on the right, with a touch of random pitch so no two presses sound quite the same.
- **Add your own:** any [Mechvibes](https://github.com/hainguyents13/mechvibes) sound pack works, old or new format. Press **Add your own…** and drop the pack's folder in.
- **Hears keys, not words.** Clackboard only learns which key position went down, never what you type. Nothing is recorded or sent anywhere. On macOS it is silent while you type a password.

## Keyboard cleaning

1. Pick how long to lock: 30 seconds to 5 minutes, or until you unlock it yourself.
2. Press **Lock keyboard to clean**. A short countdown gives you time to change your mind.
3. Wipe the keyboard. Your mouse keeps working, and sounds pause.
4. The keyboard comes back when the timer runs out, when you hold **Hold to unlock**, or when you press **Ctrl + Alt + U** (⌃⌥U on a Mac).

Turn on **Detect cleaning** and the lock starts by itself when a cloth sweeps across the keys. Shortcuts and fast typing don't set it off.

## What each system asks for

| | Sounds | Keyboard lock |
| --- | --- | --- |
| macOS | Input Monitoring for Clackboard (asked once) | The keyboard helper: your password once, then Input Monitoring for **Clackboard Keyboard Helper** |
| Windows | Nothing | Nothing |
| Linux | Read access to keyboards: `sudo usermod -aG input $USER`, then log out and in | The same |

## Build from source

Clackboard is built with [Tauri](https://tauri.app) (Rust and a web UI). You need [Node.js](https://nodejs.org) 20+ and [Rust](https://rustup.rs) (stable), plus [Tauri's system dependencies](https://tauri.app/start/prerequisites/) for your OS (on Windows, the Visual Studio C++ Build Tools).

```bash
git clone https://github.com/Conqueror102/clackboard.git
cd clackboard
npm install
npm run tauri dev
```

Release build: `npm run tauri build`. Tests: `cargo test --workspace`. On Windows, `cargo run -p keylock --example listen_selftest` checks the key listener and cleaning detection on the real keyboard hook.

| Path | What's there |
| --- | --- |
| `crates/keylock` | Keyboard access, one file per OS: the lock, the pass-through key listener (`listen`), cleaning detection and the macOS helper. |
| `src-tauri/src/sound` | The sound engine: Mechvibes pack loader (`pack.rs`) and the low-latency mixer (`engine.rs`). |
| `src-tauri/resources/packs` | The built-in sound packs. |
| `src-tauri/src/lib.rs` | The lock state machine, the system watcher and the commands the UI calls. |
| `src-tauri/src/shell.rs` | Tray icon and menu, the tray panel, the island windows. |
| `src/` | The web UI: `main.ts` (panel), `sounds.ts` (the Sounds section), `island.ts`, `overlay.ts`. |
| `design-system/` | The design system inherited from Hushboard: tokens, component CSS and icons. |

## Credits

Clackboard is a fork of [Hushboard](https://github.com/tekcify/hushboard) by Tekcify, which made the keyboard lock, panel and design system. The switch sounds come from [Mechvibes](https://github.com/hainguyents13/mechvibes) by Hải Nguyễn. Both are MIT licensed. Details in [CREDITS.md](CREDITS.md).

## Contributing

Bug reports, testing on your own hardware, new sound packs, code and design are all welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md), follow the [Code of Conduct](CODE_OF_CONDUCT.md), and report security issues privately as described in [SECURITY.md](SECURITY.md).

## License

[MIT](LICENSE) © the Clackboard contributors, and Tekcify and the Hushboard contributors. The icons and fonts keep their own licenses, listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
