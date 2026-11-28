"use client";

import Image from "next/image";
import { asset } from "@/content/site";
import { useEffect, useState } from "react";

// A counter that ticks while motion is allowed. With reduced motion it stays at `still`,
// so each illustration shows one clear frame.
function useTick(ms: number, still = 0) {
  const [tick, setTick] = useState(still);
  useEffect(() => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const id = setInterval(() => setTick((t) => t + 1), ms);
    return () => clearInterval(id);
  }, [ms]);
  return tick;
}

function Pointer({ pressed }: { pressed: boolean }) {
  return (
    <svg
      width="22"
      height="22"
      viewBox="0 0 24 24"
      className={`drop-shadow-[0_2px_4px_#0009] transition-transform duration-150 ${pressed ? "scale-90" : ""}`}
    >
      <path d="M5 3l14 7.5-6.2 1.6L10 18z" fill="#e4ebed" stroke="#101314" strokeWidth="1.4" strokeLinejoin="round" />
    </svg>
  );
}

/* ---------- Step 1: open it from the menu bar ---------- */

// 8 beats of 650ms: glide to the icon, click, panel opens, glide to the switch, click, sounds on.
export function MenuBarArt() {
  const beat = useTick(650, 6) % 8;
  const open = beat >= 2;
  const onToggle = beat >= 4;
  const pressed = beat === 1 || beat === 5;
  const on = beat >= 5;
  const pos = onToggle
    ? { left: "calc(86% - 46px)", top: "70px" }
    : beat >= 1
      ? { left: "calc(78% - 8px)", top: "10px" }
      : { left: "38%", top: "62%" };

  return (
    <div aria-hidden className="relative h-full min-h-64 w-full overflow-hidden rounded-2xl border border-line bg-surface-sunken">
      <div className="flex h-8 items-center justify-between border-b border-line bg-surface-raised/80 px-4 text-[11px] text-ink-muted">
        <span className="flex gap-4">
          <span className="font-semibold text-ink">Notes</span>
          <span>File</span>
          <span>Edit</span>
        </span>
        <span className="font-mono">9:41</span>
      </div>
      <span
        className={`absolute right-[22%] top-1.5 grid size-5 place-items-center rounded-md transition-colors ${open ? "bg-ink/15" : ""}`}
      >
        <Image src={asset("/brand.svg")} alt="" width={14} height={14} />
      </span>

      <div
        className={`absolute right-[14%] top-10 w-[min(68%,250px)] origin-top-right rounded-xl border border-line-strong/50 bg-surface-raised p-3 shadow-[0_18px_40px_#000a] transition-all duration-300 ${
          open ? "scale-100 opacity-100" : "scale-95 opacity-0"
        }`}
      >
        <div className="flex items-center justify-between">
          <span className="text-xs font-semibold">Clackboard</span>
          <span className={`flex items-center gap-1.5 text-[10px] ${on ? "text-seal" : "text-ink-muted"}`}>
            <span className={`size-1.5 rounded-full ${on ? "bg-seal" : "bg-line-strong"}`} />
            {on ? "Playing" : "Quiet"}
          </span>
        </div>
        <div className="mt-3 flex items-center justify-between rounded-lg bg-surface-sunken px-2.5 py-2 text-[11px]">
          <span>Keyboard sounds</span>
          <span className={`relative h-4 w-7 rounded-full transition-colors ${on ? "bg-seal" : "bg-line-strong/60"}`}>
            <span className={`absolute top-0.5 size-3 rounded-full bg-surface transition-all ${on ? "left-3.5" : "left-0.5"}`} />
          </span>
        </div>
        <div className="mt-2 rounded-lg border border-line px-2.5 py-1.5 text-[10px] text-ink-muted">
          <span className="font-semibold text-ink">EG Oreo</span> · Tactile
        </div>
      </div>

      <span className="absolute transition-all duration-500 ease-out" style={pos}>
        <Pointer pressed={pressed} />
      </span>
    </div>
  );
}

/* ---------- Step 2: pick a switch ---------- */

const PICKS: { name: string; kind: string; color: string }[] = [
  { name: "EG Oreo", kind: "Tactile", color: "#c9a2ff" },
  { name: "Cherry MX Blue", kind: "Clicky", color: "#5aa9ff" },
  { name: "NovelKeys Cream", kind: "Linear", color: "#ff8a8a" },
  { name: "IBM Model M", kind: "Clicky", color: "#5aa9ff" },
  { name: "Topre Purple Hybrid", kind: "Topre", color: "#ffd36e" },
];

// One switch every 1.6 s, highlighted in the list, with the island saying what's playing.
export function PickArt() {
  const active = useTick(1600, 0) % PICKS.length;
  const s = PICKS[active];
  return (
    <div aria-hidden className="flex h-full min-h-64 w-full flex-col items-center justify-center gap-5 rounded-2xl border border-line bg-surface-sunken p-5">
      <div className="flex h-10 items-center gap-3 whitespace-nowrap rounded-full bg-black px-5 text-xs font-semibold">
        <span className="size-2 rounded-full" style={{ background: s.color }} />
        <span key={s.name} className="island-letter inline-block">{s.name}</span>
        <span className="h-3 w-px bg-white/20" />
        <span className="text-ink-muted">{s.kind}</span>
      </div>
      <ul className="w-full max-w-xs overflow-hidden rounded-xl border border-line bg-surface">
        {PICKS.map((p, i) => (
          <li
            key={p.name}
            className={`flex items-center justify-between border-b border-line px-3.5 py-2 text-xs transition-colors duration-300 last:border-b-0 ${
              i === active ? "bg-suds text-on-suds" : "text-ink-muted"
            }`}
          >
            <span className="font-medium">{p.name}</span>
            <span className="text-[10px] opacity-80">{p.kind}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

/* ---------- Step 3: type anywhere ---------- */

const TYPE_ROW = ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"];

// A key goes down every 220 ms; sound arcs come off the side of the desk it sits on.
export function TypeArt() {
  const tick = useTick(220, 3);
  const key = (tick * 7) % TYPE_ROW.length;
  const left = key < TYPE_ROW.length / 2;
  return (
    <div aria-hidden className="flex h-full min-h-64 w-full flex-col items-center justify-center gap-6 rounded-2xl border border-line bg-surface-sunken p-5">
      <div className="relative w-full max-w-md">
        <div className="grid grid-cols-10 gap-[1.2%] rounded-xl bg-key-edge/60 p-[2.5%]">
          {TYPE_ROW.map((k, i) => (
            <span
              key={k}
              className={`keycap aspect-square transition-transform duration-100 ${i === key ? "translate-y-[2px] !bg-seal !text-on-seal" : ""}`}
            >
              {k}
            </span>
          ))}
        </div>
        <span key={tick} className={`sound-arcs ${left ? "left-[-14%]" : "right-[-14%] scale-x-[-1]"}`}>
          <svg viewBox="0 0 40 60" width="40" height="60">
            <path d="M30 18a14 14 0 0 0 0 24" fill="none" stroke="var(--seal)" strokeWidth="3" strokeLinecap="round" />
            <path d="M20 10a24 24 0 0 0 0 40" fill="none" stroke="var(--seal)" strokeWidth="3" strokeLinecap="round" opacity="0.6" />
          </svg>
        </span>
      </div>
      <div className="flex flex-wrap items-center justify-center gap-x-4 gap-y-2 text-xs text-ink-muted">
        <span>Press and release, every key</span>
        <span className="h-3 w-px bg-line-strong/50" />
        <span>{left ? "Left keys on the left" : "Right keys on the right"}</span>
      </div>
    </div>
  );
}

/* ---------- Step 4: wipe without typing ---------- */

const ROWS = [
  ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
  ["A", "S", "D", "F", "G", "H", "J", "K", "L", ";"],
  ["Z", "X", "C", "V", "B", "N", "M", ",", ".", "/"],
];
const SWEEP = 3.2; // seconds the cloth takes to cross, one way. See .wipe-cloth in globals.css.

export function WipeArt() {
  const tick = useTick(140, 40);
  const stopped = 12 + (tick % 160);

  return (
    <div aria-hidden className="flex h-full min-h-64 w-full flex-col items-center justify-center gap-6 rounded-2xl border border-line bg-surface-sunken p-5">
      <div className="flex h-10 items-center gap-3 whitespace-nowrap rounded-full bg-black px-5 text-xs">
        <span className="font-semibold">Locked</span>
        <span className="h-3 w-px bg-white/20" />
        <span className="tabular-nums text-on-suds">{stopped} key presses stopped</span>
      </div>

      <div className="scene-deck relative w-full max-w-md overflow-hidden rounded-xl bg-key-edge/60 p-[2.5%]">
        <div className="grid gap-[1.2%]">
          {ROWS.map((row, r) => (
            <div key={r} className="grid grid-cols-10 gap-[1.2%]" style={{ paddingLeft: `${r * 2.5}%` }}>
              {row.map((k, c) => (
                <span
                  key={k}
                  className="keycap wipe-key aspect-square"
                  // The key lights as the cloth passes over its column, on the way out and on the way back.
                  style={{ animationDelay: `${((c + 0.5 + r * 0.25) / 10) * SWEEP - 0.25}s` }}
                >
                  {k}
                </span>
              ))}
            </div>
          ))}
        </div>
        <span className="cloth wipe-cloth top-[8%]" />
      </div>
    </div>
  );
}
