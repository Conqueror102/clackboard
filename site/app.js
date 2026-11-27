// Clackboard website: the live typing demo and the download button.
// The demo plays the same Mechvibes recordings as the app (converted by tools/site-packs):
// a press sound on key down, a release sound on key up, panned by where the key sits, with a
// touch of random pitch. Like the app, it only looks at which key moved, never at what you type.

const SWITCHES = [
  { id: "eg-oreo", name: "EG Oreo", kind: "tactile" },
  { id: "cherrymx-blue-abs", name: "MX Blue", kind: "clicky" },
  { id: "ibm-model-m", name: "Model M", kind: "clicky" },
  { id: "holy-panda", name: "Holy Panda", kind: "tactile" },
  { id: "nk-cream", name: "NK Cream", kind: "linear" },
  { id: "cherrymx-red-abs", name: "MX Red", kind: "linear" },
  { id: "topre-purple-hybrid-pbt", name: "Topre", kind: "topre" },
  { id: "typewriter", name: "Typewriter", kind: "clicky" },
];
const KIND_COLOR = { clicky: "var(--clicky)", tactile: "var(--tactile)", linear: "var(--linear)", topre: "var(--topre)" };

const isMac = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

// [code, width in keys, legend, style]
const LAYOUT = [
  [["Backquote", 1, "`"], ["Digit1", 1, "1"], ["Digit2", 1, "2"], ["Digit3", 1, "3"], ["Digit4", 1, "4"], ["Digit5", 1, "5"], ["Digit6", 1, "6"], ["Digit7", 1, "7"], ["Digit8", 1, "8"], ["Digit9", 1, "9"], ["Digit0", 1, "0"], ["Minus", 1, "-"], ["Equal", 1, "="], ["Backspace", 2, "⌫", "mod"]],
  [["Tab", 1.5, "Tab", "mod"], ["KeyQ", 1, "Q"], ["KeyW", 1, "W"], ["KeyE", 1, "E"], ["KeyR", 1, "R"], ["KeyT", 1, "T"], ["KeyY", 1, "Y"], ["KeyU", 1, "U"], ["KeyI", 1, "I"], ["KeyO", 1, "O"], ["KeyP", 1, "P"], ["BracketLeft", 1, "["], ["BracketRight", 1, "]"], ["Backslash", 1.5, "\\"]],
  [["CapsLock", 1.75, "Caps", "mod"], ["KeyA", 1, "A"], ["KeyS", 1, "S"], ["KeyD", 1, "D"], ["KeyF", 1, "F"], ["KeyG", 1, "G"], ["KeyH", 1, "H"], ["KeyJ", 1, "J"], ["KeyK", 1, "K"], ["KeyL", 1, "L"], ["Semicolon", 1, ";"], ["Quote", 1, "'"], ["Enter", 2.25, "Enter", "accent"]],
  [["ShiftLeft", 2.25, "Shift", "mod"], ["KeyZ", 1, "Z"], ["KeyX", 1, "X"], ["KeyC", 1, "C"], ["KeyV", 1, "V"], ["KeyB", 1, "B"], ["KeyN", 1, "N"], ["KeyM", 1, "M"], ["Comma", 1, ","], ["Period", 1, "."], ["Slash", 1, "/"], ["ShiftRight", 2.75, "Shift", "mod"]],
  [["ControlLeft", 1.25, isMac ? "⌃" : "Ctrl", "mod"], [isMac ? "AltLeft" : "MetaLeft", 1.25, isMac ? "⌥" : "Win", "mod"], [isMac ? "MetaLeft" : "AltLeft", 1.25, isMac ? "⌘" : "Alt", "mod"], ["Space", 6.25, ""], [isMac ? "MetaRight" : "AltRight", 1.25, isMac ? "⌘" : "Alt", "mod"], [isMac ? "AltRight" : "MetaRight", 1.25, isMac ? "⌥" : "Win", "mod"], ["ContextMenu", 1.25, "☰", "mod"], ["ControlRight", 1.25, isMac ? "⌃" : "Ctrl", "mod"]],
];
const WIDTH_UNITS = 15;
const GAP = 0.12; // in key units
const PAD = 0.25;

// ---------- Keyboard ----------
const board = document.getElementById("board");
const keyEls = new Map();
const keyPan = new Map();

for (const row of LAYOUT) {
  const r = document.createElement("div");
  r.className = "row";
  let x = 0;
  for (const [code, w, label, style] of row) {
    const k = document.createElement("div");
    k.className = `key${style ? ` ${style}` : ""}`;
    k.dataset.code = code;
    k.style.width = `calc(var(--u) * ${w} + var(--u) * ${GAP} * ${w - 1})`;
    k.innerHTML = `<span>${label}</span>`;
    r.appendChild(k);
    keyEls.set(code, k);
    // Pan: where the key's centre sits, -1 (left) … 1 (right), kept off the extremes.
    keyPan.set(code, (((x + w / 2) / WIDTH_UNITS) * 2 - 1) * 0.7);
    x += w;
  }
  board.appendChild(r);
}

function fitBoard() {
  const inner = board.clientWidth - 2;
  const u = Math.min(60, inner / (WIDTH_UNITS + (WIDTH_UNITS - 1) * GAP + PAD * 2));
  board.style.setProperty("--u", `${u}px`);
}
new ResizeObserver(fitBoard).observe(board);
fitBoard();

// ---------- Sound ----------
let ctx = null;
let master = null;
const packs = new Map(); // id -> Promise<{ buffer, keys, names }>
let current = SWITCHES[0].id;
let volume = 0.7;
const status = document.getElementById("status");

function audio() {
  if (!ctx) {
    const Ctor = window.AudioContext || window.webkitAudioContext;
    if (!Ctor) return null;
    ctx = new Ctor({ latencyHint: "interactive" });
    master = ctx.createGain();
    master.gain.value = volume;
    master.connect(ctx.destination);
  }
  return ctx;
}

function loadPack(id) {
  if (!packs.has(id)) {
    const c = audio();
    const p = Promise.all([
      fetch(`packs/${id}.json`).then((r) => r.json()),
      fetch(`packs/${id}.wav`).then((r) => r.arrayBuffer()),
    ]).then(([meta, data]) =>
      new Promise((resolve, reject) => c.decodeAudioData(data, resolve, reject)).then((buffer) => ({
        buffer,
        keys: meta.keys,
        names: Object.keys(meta.keys).filter((k) => /^(Key|Digit)/.test(k)).sort(),
      })),
    );
    p.catch(() => packs.delete(id));
    packs.set(id, p);
  }
  return packs.get(id);
}

function soundFor(pack, code) {
  if (pack.keys[code]) return pack.keys[code];
  // A key the pack doesn't have: the same stand-in every time, like the app.
  let h = 5381;
  for (const ch of code) h = ((h * 33) ^ ch.charCodeAt(0)) >>> 0;
  return pack.keys[pack.names[h % pack.names.length]];
}

function playSeg(pack, seg, gain, pan, rate) {
  const src = ctx.createBufferSource();
  src.buffer = pack.buffer;
  src.playbackRate.value = rate;
  const g = ctx.createGain();
  g.gain.value = gain;
  let node = src.connect(g);
  if (ctx.createStereoPanner) {
    const p = ctx.createStereoPanner();
    p.pan.value = pan;
    node = node.connect(p);
  }
  node.connect(master);
  const start = seg[0] / 1000;
  const length = (seg[1] - seg[0]) / 1000;
  src.start(0, start, length);
}

async function key(code, down) {
  const c = audio();
  if (!c) return;
  if (c.state !== "running") c.resume();
  const pack = await loadPack(current).catch(() => null);
  if (!pack) return;
  const segs = soundFor(pack, code);
  if (!segs) return;
  const pan = keyPan.get(code) ?? 0;
  const rate = 1 + (Math.random() - 0.5) * 0.08;
  if (down) playSeg(pack, segs[0], 1, pan, rate);
  else if (segs[1]) playSeg(pack, segs[1], 0.9, pan, rate);
  else playSeg(pack, [segs[0][0], segs[0][0] + (segs[0][1] - segs[0][0]) * 0.6], 0.35, pan, rate * 1.12);
}

// ---------- Input ----------
const held = new Set();

function press(code) {
  if (held.has(code)) return; // auto-repeat
  held.add(code);
  keyEls.get(code)?.classList.add("down");
  key(code, true);
}

function release(code) {
  if (!held.delete(code)) return;
  keyEls.get(code)?.classList.remove("down");
  key(code, false);
}

window.addEventListener("keydown", (e) => {
  if (e.repeat || !e.code) return;
  if (e.target instanceof HTMLInputElement && e.target.type === "range" && e.code.startsWith("Arrow")) return;
  press(e.code);
});
window.addEventListener("keyup", (e) => e.code && release(e.code));
window.addEventListener("blur", () => [...held].forEach(release));

board.addEventListener("pointerdown", (e) => {
  const k = e.target.closest(".key");
  if (!k) return;
  e.preventDefault();
  k.setPointerCapture?.(e.pointerId);
  press(k.dataset.code);
  const up = () => {
    release(k.dataset.code);
    k.removeEventListener("pointerup", up);
    k.removeEventListener("pointercancel", up);
  };
  k.addEventListener("pointerup", up);
  k.addEventListener("pointercancel", up);
});

// ---------- Controls ----------
const picker = document.getElementById("switches-picker");
picker.innerHTML = SWITCHES.map(
  (s) => `<button class="chip" type="button" role="radio" aria-checked="${s.id === current}" data-id="${s.id}"><i style="background:${KIND_COLOR[s.kind]}"></i>${s.name}</button>`,
).join("");

async function choose(id) {
  current = id;
  const s = SWITCHES.find((x) => x.id === id);
  for (const b of picker.querySelectorAll(".chip")) b.setAttribute("aria-checked", String(b.dataset.id === id));
  const chip = picker.querySelector(`[data-id="${id}"]`);
  chip.dataset.loading = "true";
  status.textContent = `Loading ${s.name}…`;
  try {
    await loadPack(id);
    if (current === id) status.textContent = `${s.name}: ${s.kind}. Type away.`;
  } catch {
    if (current === id) status.textContent = `Couldn't load ${s.name}. Check your connection and try again.`;
  } finally {
    delete chip.dataset.loading;
  }
}

picker.addEventListener("click", (e) => {
  const b = e.target.closest(".chip");
  if (!b) return;
  audio()?.resume();
  choose(b.dataset.id).then(() => {
    // A short phrase so the switch can be heard right away.
    const phrase = ["KeyC", "KeyL", "KeyA", "KeyC", "KeyK", "Space", "KeyC", "KeyL", "KeyA", "KeyC", "KeyK"];
    phrase.forEach((code, i) => {
      setTimeout(() => press(code), i * 95);
      setTimeout(() => release(code), i * 95 + 60);
    });
  });
});

document.getElementById("volume").addEventListener("input", (e) => {
  volume = Number(e.target.value) / 100;
  if (master) master.gain.value = volume;
});

// Fetch the first switch once the page is idle, so the first key already sounds.
(window.requestIdleCallback || ((f) => setTimeout(f, 600)))(() => choose(current));

// ---------- Downloads ----------
// Every button starts as a link to the release page; once the latest release is known, each
// one points straight at its file, so a click starts the download.
const ua = navigator.userAgent;
const mobile = /Android|iPhone|iPad|iPod/.test(ua);
const os = mobile
  ? null
  : /Mac/.test(navigator.platform || ua)
    ? "mac"
    : /Win/.test(navigator.platform || ua)
      ? "win"
      : /Linux|X11|CrOS/.test(ua)
        ? "linux"
        : null;
const OS_NAME = { mac: "macOS", win: "Windows", linux: "Linux" };
// Which file each button gets. Linux's main button picks the .deb (most desktops).
const ASSETS = {
  dmg: /_universal\.dmg$/,
  exe: /_x64-setup\.exe$/,
  deb: /_amd64\.deb$/,
  rpm: /\.x86_64\.rpm$/,
  appimage: /_amd64\.AppImage$/,
};
const MAIN_ASSET = { mac: "dmg", win: "exe", linux: "deb" };

if (os) {
  document.getElementById("download-label").textContent = `Download for ${OS_NAME[os]}`;
  document.querySelector(`.platform[data-os="${os}"]`)?.classList.add("is-you");
  document.getElementById("download-note").innerHTML = `Free forever. Also for ${Object.entries(OS_NAME)
    .filter(([k]) => k !== os)
    .map(([, v]) => v)
    .join(" and ")}: <a href="#download">all downloads</a>.`;
} else {
  // Phones and tablets can't run it: send people to the download list instead.
  const main = document.getElementById("download-main");
  main.href = "#download";
  document.getElementById("download-label").textContent = "Get it for your computer";
}

const mb = (bytes) => `${Math.round(bytes / 1048576)} MB`;

fetch("https://api.github.com/repos/Conqueror102/clackboard/releases/latest", {
  headers: { Accept: "application/vnd.github+json" },
})
  .then((r) => (r.ok ? r.json() : Promise.reject(new Error(`HTTP ${r.status}`))))
  .then((release) => {
    const find = (key) => release.assets.find((a) => ASSETS[key].test(a.name));
    for (const link of document.querySelectorAll("[data-asset]")) {
      const asset = find(link.dataset.asset);
      if (!asset) continue;
      link.href = asset.browser_download_url;
      link.setAttribute("download", "");
      link.title = `${asset.name} (${mb(asset.size)})`;
    }
    if (os) {
      const asset = find(MAIN_ASSET[os]);
      if (asset) {
        const main = document.getElementById("download-main");
        main.href = asset.browser_download_url;
        main.title = asset.name;
        document.getElementById("download-label").innerHTML =
          `Download for ${OS_NAME[os]} <small>${release.tag_name} · ${mb(asset.size)}</small>`;
      }
    }
    const line = document.getElementById("release-line");
    if (line) {
      line.innerHTML = `Latest: <b>${release.tag_name}</b> · <a href="${release.html_url}">release notes, all files and checksums</a>`;
    }
  })
  .catch(() => {
    // Offline or rate-limited: the buttons keep pointing at the release page.
  });
