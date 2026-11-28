import { kindLabel, links, switches, type Kind } from "@/content/site";
import { Reveal } from "./Reveal";

const KINDS: { kind: Kind; blurb: string; color: string }[] = [
  { kind: "clicky", blurb: "A sharp click at the bump. Loud, bright, unmistakable.", color: "#5aa9ff" },
  { kind: "tactile", blurb: "A soft bump you can feel, a rounder sound.", color: "#c9a2ff" },
  { kind: "linear", blurb: "Smooth all the way down. Deep, creamy thocks.", color: "#ff8a8a" },
  { kind: "topre", blurb: "Rubber dome and spring. A muted, rounded thock.", color: "#ffd36e" },
];

// Every switch in the app, grouped by how it feels.
export function Switches() {
  return (
    <section id="switches" aria-labelledby="switches-title" className="mx-auto max-w-6xl scroll-mt-24 px-5 py-16 sm:py-24">
      <div className="grid gap-10 lg:grid-cols-[300px_1fr] lg:gap-14">
        <Reveal className="lg:sticky lg:top-28 lg:self-start">
          <p className="text-xs font-semibold uppercase tracking-[0.14em] text-seal">The switches</p>
          <h2 id="switches-title" className="mt-4 text-[clamp(2rem,4.6vw,3.2rem)] font-semibold leading-[1.05] tracking-[-0.045em]">
            26 switches, built in
          </h2>
          <p className="mt-4 max-w-sm text-ink-muted">
            Recorded from real keyboards: every key, going down and coming back up. Want another? Any{" "}
            <a className="text-ink underline decoration-line-strong underline-offset-4 hover:decoration-seal" href={links.mechvibes}>
              Mechvibes
            </a>{" "}
            sound pack drops straight in.
          </p>
        </Reveal>

        <div className="grid gap-5 sm:grid-cols-2">
          {KINDS.map(({ kind, blurb, color }, k) => {
            const list = switches.filter((s) => s.kind === kind);
            return (
              <Reveal key={kind} variant="scale" delay={k * 90} className="rounded-[28px] border border-line bg-surface-raised/40 p-6">
                <div className="flex items-center gap-2.5">
                  <span className="size-2.5 rounded-full" style={{ background: color }} />
                  <h3 className="text-lg font-semibold tracking-tight">{kindLabel[kind]}</h3>
                  <span className="ml-auto font-mono text-xs text-ink-muted">{list.length}</span>
                </div>
                <p className="mt-2 text-sm text-ink-muted">{blurb}</p>
                <ul className="mt-5 divide-y divide-line border-t border-line">
                  {list.map((s) => (
                    <li key={s.name} className="flex items-baseline justify-between gap-4 py-2.5">
                      <span className="font-medium">{s.name}</span>
                      <span className="text-right text-xs text-ink-muted">{s.note}</span>
                    </li>
                  ))}
                </ul>
              </Reveal>
            );
          })}
        </div>
      </div>
    </section>
  );
}
