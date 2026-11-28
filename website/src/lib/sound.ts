// The typing demo's sound engine. It plays the same recordings as the app (converted by
// tools/site-packs): a press clip on key down, a release clip on key up, panned by where the key
// sits on the keyboard, with a touch of random pitch. Browsers only allow sound after a click
// or a key press, so nothing plays until the visitor does something.
import { asset } from "@/content/site";

type Segment = [number, number]; // ms in the pack's audio
interface Pack {
  buffer: AudioBuffer;
  keys: Record<string, Segment[]>;
  names: string[];
}

let ctx: AudioContext | null = null;
let master: GainNode | null = null;
let volume = 0.7;
const packs = new Map<string, Promise<Pack>>();

function audio(): AudioContext | null {
  if (typeof window === "undefined") return null;
  if (!ctx) {
    const Ctor = window.AudioContext ?? (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!Ctor) return null;
    ctx = new Ctor({ latencyHint: "interactive" });
    master = ctx.createGain();
    master.gain.value = volume;
    master.connect(ctx.destination);
  }
  return ctx;
}

/** True once the browser lets the page make sound. */
export function soundReady(): boolean {
  return ctx?.state === "running";
}

/** Call from a click or key handler: browsers start audio only from a user gesture. */
export function unlockSound(): void {
  const c = audio();
  if (c && c.state !== "running") void c.resume();
}

export function setVolume(v: number): void {
  volume = v;
  if (master) master.gain.value = v;
}

export function loadPack(id: string): Promise<Pack> {
  let p = packs.get(id);
  if (!p) {
    const c = audio();
    if (!c) return Promise.reject(new Error("no Web Audio"));
    p = Promise.all([
      fetch(asset(`/packs/${id}.json`)).then((r) => r.json() as Promise<{ keys: Record<string, Segment[]> }>),
      fetch(asset(`/packs/${id}.wav`)).then((r) => r.arrayBuffer()),
    ]).then(([meta, data]) =>
      new Promise<AudioBuffer>((resolve, reject) => c.decodeAudioData(data, resolve, reject)).then((buffer) => ({
        buffer,
        keys: meta.keys,
        names: Object.keys(meta.keys)
          .filter((k) => /^(Key|Digit)/.test(k))
          .sort(),
      })),
    );
    p.catch((e) => {
      console.warn("clackboard: could not load", id, e);
      packs.delete(id);
    });
    packs.set(id, p);
  }
  return p;
}

function soundFor(pack: Pack, code: string): Segment[] | undefined {
  if (pack.keys[code]) return pack.keys[code];
  // A key the pack doesn't have gets the same stand-in every time, like the app.
  let h = 5381;
  for (const ch of code) h = ((h * 33) ^ ch.charCodeAt(0)) >>> 0;
  return pack.names.length ? pack.keys[pack.names[h % pack.names.length]] : undefined;
}

function play(pack: Pack, seg: Segment, gain: number, pan: number, rate: number) {
  if (!ctx || !master) return;
  const src = ctx.createBufferSource();
  src.buffer = pack.buffer;
  src.playbackRate.value = rate;
  const g = ctx.createGain();
  g.gain.value = gain;
  src.connect(g);
  let out: AudioNode = g;
  if (ctx.createStereoPanner) {
    const p = ctx.createStereoPanner();
    p.pan.value = pan;
    g.connect(p);
    out = p;
  }
  out.connect(master);
  src.start(0, seg[0] / 1000, (seg[1] - seg[0]) / 1000);
}

/** Play `code` going down or up with the given pack. `pan` is -1 (left) … 1 (right). */
export async function keySound(packId: string, code: string, down: boolean, pan: number): Promise<void> {
  const c = audio();
  if (!c || c.state !== "running") return;
  const pack = await loadPack(packId).catch(() => null);
  if (!pack) return;
  const segs = soundFor(pack, code);
  if (!segs) return;
  const rate = 1 + (Math.random() - 0.5) * 0.08;
  if (down) play(pack, segs[0], 1, pan, rate);
  else if (segs[1]) play(pack, segs[1], 0.9, pan, rate);
  else play(pack, [segs[0][0], segs[0][0] + (segs[0][1] - segs[0][0]) * 0.6], 0.35, pan, rate * 1.12);
}
