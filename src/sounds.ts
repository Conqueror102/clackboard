// The "Sounds" section of the panel: keyboard sounds on/off, the switch (sound pack), volume
// and the small touches (release sound, stereo, pitch). The sounds themselves play in the app
// (Rust), so they keep working while the panel is closed.
import { api, type SoundInfo, type SoundPrefs } from "./api";
import { icon, type IconName } from "./icons";

const $ = (id: string) => document.getElementById(id) as HTMLElement;
const escapeHtml = (s: string) => s.replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);

let info: SoundInfo | null = null;
let packsOpen = false;

const TOGGLES: { key: "release" | "stereo" | "varyPitch"; label: string; icon: IconName }[] = [
  { key: "release", label: "Sound on release too", icon: "keyboard" },
  { key: "stereo", label: "Stereo, like a real desk", icon: "laptop" },
  { key: "varyPitch", label: "Vary the pitch a little", icon: "sparkles" },
];

/** "CherryMX Blue - ABS keycaps" → "CherryMX Blue" + "ABS keycaps". */
function split(name: string): [string, string] {
  const i = name.indexOf(" - ");
  return i > 0 ? [name.slice(0, i), name.slice(i + 3)] : [name, ""];
}

function switchRow(key: string, label: string, name: IconName, on: boolean) {
  return `<label class="app__setting" for="snd-${key}">${icon(name)}<span>${label}</span>
    <button class="hb-switch" id="snd-${key}" type="button" role="switch" aria-checked="${on}" data-snd="${key}"></button></label>`;
}

function render() {
  const box = $("sounds");
  const note = $("sounds-note");
  if (!info) return;
  const p = info.prefs;
  const current = info.packs.find((k) => k.id === p.pack);

  if (!info.supported) {
    box.innerHTML = "";
    note.textContent = "Keyboard sounds are coming to this system in a later version.";
    note.hidden = false;
    return;
  }

  let html = `<div class="app__settings">${switchRow("enabled", "Keyboard sounds", "keyboard", p.enabled)}</div>`;
  if (p.enabled) {
    const [title, sub] = split(current?.name ?? "Choose a switch");
    html += `<div class="app__settings app__island-card">
      <button class="app__disclosure" id="snd-packs-toggle" type="button" aria-expanded="${packsOpen}" aria-controls="snd-packs">
        <span><span class="app__disclosure-title">${escapeHtml(title)}</span><span class="app__disclosure-meta">${escapeHtml(sub || "Switch")} · ${info.packs.length} switches</span></span>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 6 6 6-6 6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
      </button>
      <div class="hb-devices app__packs" id="snd-packs" role="radiogroup" aria-label="Switch sound" ${packsOpen ? "" : "hidden"}>
        ${info.packs
          .map((k) => {
            const on = k.id === p.pack;
            const [t, s] = split(k.name);
            const meta = [s, k.hasRelease ? "press and release" : "press", k.builtin ? "" : "your pack"].filter(Boolean).join(" · ");
            return `<button class="hb-device" type="button" role="radio" aria-checked="${on}" data-pack="${escapeHtml(k.id)}">
              <span class="hb-device__icon">${icon("keyboard")}</span>
              <span><span class="hb-device__name">${escapeHtml(t)}</span><br><span class="hb-device__meta">${escapeHtml(meta)}</span></span>
              <span></span></button>`;
          })
          .join("")}
      </div>
    </div>`;
    const vol = Math.round(p.volume * 100);
    html += `<div class="app__settings app__island-card"><div class="app__range">
        <label for="snd-volume">Volume</label><span class="app__range-value" id="snd-volume-v">${vol}%</span>
        <input class="hb-range" id="snd-volume" type="range" min="0" max="150" step="5" value="${vol}" style="--p:${(vol / 150) * 100}%" />
      </div></div>`;
    html += `<div class="app__settings">${TOGGLES.map((t) => switchRow(t.key, t.label, t.icon, p[t.key])).join("")}</div>`;
    html += `<div class="hb-row app__island-actions">
        <button class="hb-btn hb-btn--secondary" type="button" id="snd-demo">Play a typing demo</button>
        <button class="hb-btn hb-btn--ghost" type="button" id="snd-folder">Add your own…</button>
      </div>`;
  }
  box.innerHTML = html;

  if (info.error) {
    note.textContent = info.error;
    note.hidden = false;
  } else if (p.enabled) {
    note.textContent = "Clackboard hears which keys you press, never what you type. Sounds pause while the keyboard is locked.";
    note.hidden = false;
  } else {
    note.hidden = true;
  }
}

let saving: Promise<void> = Promise.resolve();

function update(change: Partial<SoundPrefs>, rerender = true) {
  if (!info) return;
  info = { ...info, prefs: { ...info.prefs, ...change } };
  if (rerender) render();
  const prefs = info.prefs;
  // One save at a time, in order; the reply says what's really running (and any error).
  saving = saving.then(async () => {
    const next = await api.setSound(prefs).catch(() => null);
    if (next && info && next.prefs.pack === info.prefs.pack) {
      const changed = next.error !== info.error || next.listening !== info.listening;
      info = { ...next, prefs: { ...next.prefs, volume: info.prefs.volume } };
      if (changed) render();
    }
  });
}

export async function initSounds() {
  info = await api.getSound().catch(() => null);
  render();

  const box = $("sounds");
  box.addEventListener("click", (e) => {
    const t = e.target as HTMLElement;
    const sw = t.closest<HTMLButtonElement>("button[data-snd]");
    if (sw && info) {
      const key = sw.dataset.snd as "enabled" | "release" | "stereo" | "varyPitch";
      update({ [key]: !info.prefs[key] });
      return;
    }
    if (t.closest("#snd-packs-toggle")) {
      packsOpen = !packsOpen;
      render();
      return;
    }
    const pack = t.closest<HTMLButtonElement>("button[data-pack]");
    if (pack) {
      update({ pack: pack.dataset.pack! });
      // Let people hear the switch they just picked.
      void saving.then(() => api.soundDemo());
      return;
    }
    if (t.closest("#snd-demo")) void api.soundDemo();
    if (t.closest("#snd-folder")) void api.openPacksFolder();
  });

  box.addEventListener("input", (e) => {
    const input = e.target as HTMLInputElement;
    if (input.id !== "snd-volume") return;
    const v = Number(input.value);
    input.style.setProperty("--p", `${(v / 150) * 100}%`);
    $("snd-volume-v").textContent = `${v}%`;
    update({ volume: v / 100 }, false);
  });

  // Coming back to the panel after adding a pack to the folder: show it.
  window.addEventListener("focus", async () => {
    const next = await api.getSound().catch(() => null);
    if (next && info && next.packs.length !== info.packs.length) {
      info = next;
      render();
    }
  });
}
