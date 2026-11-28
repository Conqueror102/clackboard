"use client";

import Image from "next/image";
import { useCallback, useEffect, useRef, useState } from "react";
import { asset, kindLabel, switches, type Kind } from "@/content/site";
import { keySound, loadPack, setVolume, unlockSound } from "@/lib/sound";
import { BatteryGlyph, ControlCentreGlyph, DOCK, SearchGlyph, TrashIcon, WifiGlyph } from "./MacIcons";
import "./MacScene.css";

// A MacBook you can type on. Until the visitor does anything it types a few lines by itself,
// silently; the first click or key press turns the sound on and hands it the real keyboard.
// Keys light up where they sit, the text lands in the window, and the island in the notch
// shows which switch is playing. Like the app, it only uses which key moved.

// [label, width, kind, code]. Widths are relative within a row, like real keycaps.
type Key = [string, number, ("fn" | "small" | "touch" | "arrows")?, string?];
const ROWS: Key[][] = [
  [["esc", 1.5, "fn", "Escape"], ...Array.from({ length: 12 }, (_, i): Key => [`F${i + 1}`, 1, "fn", `F${i + 1}`]), ["", 1, "touch"]],
  [["`", 1, undefined, "Backquote"], ["1", 1, undefined, "Digit1"], ["2", 1, undefined, "Digit2"], ["3", 1, undefined, "Digit3"], ["4", 1, undefined, "Digit4"], ["5", 1, undefined, "Digit5"], ["6", 1, undefined, "Digit6"], ["7", 1, undefined, "Digit7"], ["8", 1, undefined, "Digit8"], ["9", 1, undefined, "Digit9"], ["0", 1, undefined, "Digit0"], ["-", 1, undefined, "Minus"], ["=", 1, undefined, "Equal"], ["delete", 1.5, "small", "Backspace"]],
  [["tab", 1.5, "small", "Tab"], ["Q", 1, undefined, "KeyQ"], ["W", 1, undefined, "KeyW"], ["E", 1, undefined, "KeyE"], ["R", 1, undefined, "KeyR"], ["T", 1, undefined, "KeyT"], ["Y", 1, undefined, "KeyY"], ["U", 1, undefined, "KeyU"], ["I", 1, undefined, "KeyI"], ["O", 1, undefined, "KeyO"], ["P", 1, undefined, "KeyP"], ["[", 1, undefined, "BracketLeft"], ["]", 1, undefined, "BracketRight"], ["\\", 1, undefined, "Backslash"]],
  [["caps", 1.8, "small", "CapsLock"], ["A", 1, undefined, "KeyA"], ["S", 1, undefined, "KeyS"], ["D", 1, undefined, "KeyD"], ["F", 1, undefined, "KeyF"], ["G", 1, undefined, "KeyG"], ["H", 1, undefined, "KeyH"], ["J", 1, undefined, "KeyJ"], ["K", 1, undefined, "KeyK"], ["L", 1, undefined, "KeyL"], [";", 1, undefined, "Semicolon"], ["'", 1, undefined, "Quote"], ["return", 1.7, "small", "Enter"]],
  [["shift", 2.3, "small", "ShiftLeft"], ["Z", 1, undefined, "KeyZ"], ["X", 1, undefined, "KeyX"], ["C", 1, undefined, "KeyC"], ["V", 1, undefined, "KeyV"], ["B", 1, undefined, "KeyB"], ["N", 1, undefined, "KeyN"], ["M", 1, undefined, "KeyM"], [",", 1, undefined, "Comma"], [".", 1, undefined, "Period"], ["/", 1, undefined, "Slash"], ["shift", 2.3, "small", "ShiftRight"]],
  [["fn", 1, "small", "Fn"], ["⌃", 1, undefined, "ControlLeft"], ["⌥", 1, undefined, "AltLeft"], ["⌘", 1.25, undefined, "MetaLeft"], ["space", 5.2, "small", "Space"], ["⌘", 1.25, undefined, "MetaRight"], ["⌥", 1, undefined, "AltRight"], ["◀", 1, undefined, "ArrowLeft"], ["▲▼", 1, "arrows"], ["▶", 1, undefined, "ArrowRight"]],
];

// Where each key sits across the keyboard, for stereo: -1 (left) … 1 (right), kept off the edges.
const PAN = new Map<string, number>();
for (const row of ROWS) {
  const total = row.reduce((sum, [, w]) => sum + w, 0);
  let x = 0;
  for (const [, w, kind, code] of row) {
    const pan = (((x + w / 2) / total) * 2 - 1) * 0.7;
    if (code) PAN.set(code, pan);
    if (kind === "arrows") {
      PAN.set("ArrowUp", pan);
      PAN.set("ArrowDown", pan);
    }
    x += w;
  }
}

const DEMO = switches.filter((s): s is typeof s & { demo: string } => Boolean(s.demo));
const KIND_COLOR: Record<Kind, string> = { clicky: "#5aa9ff", tactile: "#c9a2ff", linear: "#ff8a8a", topre: "#ffd36e" };

// What the MacBook types by itself before anyone touches it.
const PHRASES = ["Every key, a real switch.", "Clicky, tactile, linear or Topre.", "Your keyboard, finally audible."];
const CHAR_CODE: Record<string, string> = { " ": "Space", ",": "Comma", ".": "Period", "'": "Quote", "-": "Minus" };
const codeFor = (ch: string) =>
  CHAR_CODE[ch] ?? (/[a-z]/i.test(ch) ? `Key${ch.toUpperCase()}` : /\d/.test(ch) ? `Digit${ch}` : "Space");

export function KeyboardScene() {
  const [live, setLive] = useState(false);
  const [pack, setPack] = useState(DEMO[0].demo);
  const [text, setText] = useState("");
  const [panelOpen, setPanelOpen] = useState(false);
  const [pulse, setPulse] = useState(0);
  const [loading, setLoading] = useState(false);
  const [vol, setVol] = useState(70);

  const keyRefs = useRef<Record<string, HTMLSpanElement | null>>({});
  const held = useRef(new Set<string>());
  const liveRef = useRef(false);
  const packRef = useRef(pack);
  const sceneRef = useRef<HTMLElement>(null);
  const inView = useRef(true);

  const current = DEMO.find((s) => s.demo === pack) ?? DEMO[0];

  const light = (code: string, on: boolean) => {
    keyRefs.current[code]?.classList.toggle("on", on);
  };

  const goLive = useCallback(() => {
    unlockSound();
    if (!liveRef.current) {
      liveRef.current = true;
      setLive(true);
      setText("");
    }
  }, []);

  const press = useCallback((code: string, ch: string | null) => {
    if (held.current.has(code)) return; // auto-repeat
    held.current.add(code);
    light(code, true);
    setPulse((p) => p + 1);
    void keySound(packRef.current, code, true, PAN.get(code) ?? 0);
    setText((t) => {
      if (code === "Backspace") return t.slice(0, -1);
      if (code === "Enter") return `${t}\n`.slice(-160);
      return ch ? (t + ch).slice(-160) : t;
    });
  }, []);

  const release = useCallback((code: string) => {
    if (!held.current.delete(code)) return;
    light(code, false);
    void keySound(packRef.current, code, false, PAN.get(code) ?? 0);
  }, []);

  const choose = useCallback(
    (id: string) => {
      goLive();
      packRef.current = id;
      setPack(id);
      setLoading(true);
      loadPack(id)
        .catch(() => {})
        .finally(() => {
          setLoading(false);
          // A short phrase, so the new switch is heard straight away.
          "clack clack".split("").forEach((ch, i) => {
            setTimeout(() => press(codeFor(ch), ch), i * 90);
            setTimeout(() => release(codeFor(ch)), i * 90 + 55);
          });
        });
    },
    [goLive, press, release],
  );

  // Fetch the first switch early, so the first key already sounds.
  useEffect(() => {
    const t = setTimeout(() => void loadPack(DEMO[0].demo).catch(() => {}), 1200);
    return () => clearTimeout(t);
  }, []);

  // The page's own keyboard: every key goes to the MacBook once it's live.
  useEffect(() => {
    const io = new IntersectionObserver(([e]) => (inView.current = e.isIntersecting), { threshold: 0.2 });
    if (sceneRef.current) io.observe(sceneRef.current);

    const typing = (t: EventTarget | null) =>
      t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement || t instanceof HTMLSelectElement;
    const down = (e: KeyboardEvent) => {
      if (e.repeat || !e.code || typing(e.target)) return;
      if (!liveRef.current) {
        if (!inView.current) return;
        goLive();
      }
      // While typing on the demo, Space and the quick-find keys shouldn't scroll or search.
      if (inView.current && !e.ctrlKey && !e.metaKey && ["Space", "Quote", "Slash"].includes(e.code)) e.preventDefault();
      press(e.code, e.key.length === 1 && !e.ctrlKey && !e.metaKey ? e.key : null);
    };
    const up = (e: KeyboardEvent) => {
      if (e.code) release(e.code);
    };
    const blur = () => [...held.current].forEach(release);
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    window.addEventListener("blur", blur);
    return () => {
      io.disconnect();
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
      window.removeEventListener("blur", blur);
    };
  }, [goLive, press, release]);

  // The self-typing intro: silent, and it stops for good once the visitor takes over.
  useEffect(() => {
    if (live) return;
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      setText(PHRASES[0]);
      return;
    }
    let phrase = 0;
    let i = 0;
    let lit: string | null = null;
    const id = setInterval(() => {
      if (lit) {
        light(lit, false);
        lit = null;
      }
      const p = PHRASES[phrase];
      if (i < p.length) {
        const ch = p[i++];
        lit = codeFor(ch);
        light(lit, true);
        setPulse((n) => n + 1);
        setText(p.slice(0, i));
      } else if (i < p.length + 14) {
        i++; // linger on the finished line
      } else {
        phrase = (phrase + 1) % PHRASES.length;
        i = 0;
        setText("");
      }
    }, 95);
    return () => {
      clearInterval(id);
      if (lit) light(lit, false);
    };
  }, [live]);

  const tapKey = (code: string | undefined) => (e: React.PointerEvent) => {
    if (!code) return;
    e.preventDefault();
    goLive();
    press(code, null);
    const up = () => {
      release(code);
      window.removeEventListener("pointerup", up);
      window.removeEventListener("pointercancel", up);
    };
    window.addEventListener("pointerup", up);
    window.addEventListener("pointercancel", up);
  };

  const keyEl = (code: string | undefined, label: string, cls: string, flex?: number, k?: string) => (
    <span
      key={k ?? code ?? label}
      ref={(el) => {
        if (code) keyRefs.current[code] = el;
      }}
      className={cls}
      style={flex ? { flex } : undefined}
      onPointerDown={tapKey(code)}
    >
      {label}
    </span>
  );

  return (
    <div className="relative z-10 mx-auto w-full max-w-[56rem]">
      <div aria-hidden className="absolute -inset-x-16 -top-10 bottom-10 -z-10 bg-[radial-gradient(ellipse_at_50%_35%,#2ee6a82a,transparent_62%)] blur-2xl" />
      <figure
        ref={sceneRef}
        data-cursor-grid-ignore
        className="mac"
        aria-label="A MacBook you can type on: press keys to hear the selected switch"
        onPointerDown={() => goLive()}
      >
        <div className="mac-lid">
          <div className="mac-screen">
            <div className="mac-view">
              <div className="mac-menubar">
                <span className="left">
                  <b>Notes</b>
                  <span>File</span>
                  <span>Edit</span>
                  <span className="hidden sm:inline">Format</span>
                  <span className="hidden sm:inline">View</span>
                </span>
                <span className="right">
                  <button
                    type="button"
                    className="mac-tray"
                    data-open={panelOpen}
                    aria-label="Choose a switch"
                    aria-expanded={panelOpen}
                    onClick={() => setPanelOpen((o) => !o)}
                  >
                    <Image src={asset("/brand.svg")} alt="" width={14} height={14} className="size-[calc(var(--u)*1.35)]" />
                  </button>
                  <span className="battery">
                    <BatteryGlyph />
                  </span>
                  <WifiGlyph />
                  <SearchGlyph />
                  <ControlCentreGlyph />
                  <span>Fri 28 Nov&nbsp;&nbsp;9:41</span>
                </span>
              </div>

              <div className="mac-window">
                <header>
                  <i />
                  <i />
                  <i />
                </header>
                <div className="doc">
                  <h4>{live ? "Your turn" : "Clackboard"}</h4>
                  <p className="typed">
                    {text}
                    <span className="caret" />
                  </p>
                  {!live && (
                    <>
                      <div className="line w-[80%]" />
                      <div className="line w-[64%]" />
                    </>
                  )}
                </div>
              </div>

              <div className="mac-dock">
                {DOCK.map((app) => (
                  <span key={app.name} className="app" data-running={app.running ?? false}>
                    {app.icon}
                  </span>
                ))}
                <span className="app" data-running>
                  <Image src={asset("/brand.svg")} alt="" width={64} height={64} />
                </span>
                <span className="sep" />
                <span className="app">
                  <TrashIcon />
                </span>
              </div>

              <div className="mac-panel mac-switches" data-open={panelOpen}>
                <div className="row">
                  <b>Clackboard</b>
                  <span className="flex items-center gap-[calc(var(--u)*0.5)] text-ink-muted">
                    <span className="size-[calc(var(--u)*0.6)] rounded-full bg-seal" />
                    Sounds on
                  </span>
                </div>
                <ul>
                  {DEMO.map((s) => (
                    <li key={s.demo}>
                      <button
                        type="button"
                        data-on={s.demo === pack}
                        onClick={(e) => {
                          e.stopPropagation();
                          choose(s.demo);
                          setPanelOpen(false);
                        }}
                      >
                        <span>{s.name}</span>
                        <small>{kindLabel[s.kind]}</small>
                      </button>
                    </li>
                  ))}
                </ul>
              </div>

              <div role="status" className="mac-island" data-state="playing">
                <div className="pane bar" data-show="true">
                  <span className="now">
                    <i style={{ background: KIND_COLOR[current.kind] }} />
                    {loading ? "Loading…" : current.name}
                  </span>
                  <span className="eq" aria-hidden key={pulse}>
                    <b />
                    <b />
                    <b />
                    <b />
                  </span>
                </div>
                <span className="cam" />
              </div>

              {!live && (
                <button type="button" className="mac-hint" onClick={goLive}>
                  <svg viewBox="0 0 24 24" aria-hidden>
                    <path d="M4 9v6h4l5 4V5L8 9H4Z" fill="currentColor" />
                    <path d="M16 9a4 4 0 0 1 0 6M18.5 6.5a7.5 7.5 0 0 1 0 11" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
                  </svg>
                  Click, then type to hear it
                </button>
              )}
            </div>
          </div>
        </div>

        <div className="mac-hinge" />
        <div className="mac-base" aria-hidden>
          <div className="mac-deck">
            <div className="mac-top">
              <div className="mac-grille" />
              <div className="mac-keys">
                {ROWS.map((row, r) => (
                  <div key={r} className="mac-row">
                    {row.map(([label, w, kind, code], i) =>
                      kind === "arrows" ? (
                        <span key="arrows" className="flex flex-col justify-between" style={{ flex: w }}>
                          {keyEl("ArrowUp", "▲", "mkey half")}
                          {keyEl("ArrowDown", "▼", "mkey half")}
                        </span>
                      ) : (
                        keyEl(code, label, `mkey ${kind ?? ""}`, w, `${r}-${i}`)
                      ),
                    )}
                  </div>
                ))}
              </div>
              <div className="mac-grille" />
            </div>
            <div className="mac-trackpad" />
            <div className="lip" />
          </div>
        </div>
      </figure>

      <div className="mt-8 flex flex-col items-center gap-4">
        <div role="radiogroup" aria-label="Switch" className="flex flex-wrap justify-center gap-2">
          {DEMO.map((s) => (
            <button
              key={s.demo}
              type="button"
              role="radio"
              aria-checked={s.demo === pack}
              onClick={() => choose(s.demo)}
              className={`inline-flex h-9 items-center gap-2 rounded-full border px-3.5 text-sm font-medium transition-colors ${
                s.demo === pack ? "border-seal bg-suds text-on-suds" : "border-line bg-surface-raised/60 text-ink-muted hover:text-ink"
              }`}
            >
              <span className="size-2 rounded-full" style={{ background: KIND_COLOR[s.kind] }} />
              {s.name}
            </button>
          ))}
        </div>
        <label className="flex items-center gap-3 text-xs text-ink-muted">
          Volume
          <input
            type="range"
            min={0}
            max={100}
            value={vol}
            onChange={(e) => {
              const v = Number(e.target.value);
              setVol(v);
              setVolume(v / 100);
            }}
            className="w-32 accent-[var(--seal)]"
          />
        </label>
      </div>
    </div>
  );
}
