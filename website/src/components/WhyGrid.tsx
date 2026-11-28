import { Reveal } from "./Reveal";

const reasons: { title: string; body: string; soon?: boolean }[] = [
  {
    title: "Real recordings",
    body: "Every sound is a recorded switch, not a synth. The IBM Model M even has each key recorded on its own.",
  },
  {
    title: "Down and up",
    body: "A press sound when the key bottoms out and a release sound when it comes back, like the real thing.",
  },
  {
    title: "Stereo desk",
    body: "Keys on the left sound on the left, keys on the right on the right. Headphones make it obvious.",
  },
  {
    title: "Never machine-made",
    body: "A touch of random pitch on every press, so fast typing sounds like hands, not a drum loop.",
  },
  {
    title: "Hears keys, not words",
    body: "Clackboard only knows which key position moved. Nothing you type is read, saved or sent anywhere.",
  },
  {
    title: "Quiet when it matters",
    body: "On a Mac it goes silent in password fields. Sounds pause while the keyboard is locked for cleaning.",
  },
  {
    title: "Bring your own",
    body: "Drop in any Mechvibes sound pack, old format or new, and it shows up in the list.",
  },
  {
    title: "Pauses on calls",
    body: "Going quiet while another app uses the microphone, and in apps you choose.",
    soon: true,
  },
];

export function WhyGrid() {
  return (
    <section id="features" aria-labelledby="why-title" className="mx-auto max-w-6xl scroll-mt-24 px-5 py-16 sm:py-24">
      <Reveal as="h2" id="why-title" className="text-center text-[clamp(2rem,4.6vw,3.4rem)] font-semibold tracking-[-0.045em]">
        Why Clackboard
      </Reveal>
      <Reveal as="ul" variant="scale" delay={120} className="mt-12 grid gap-px overflow-hidden rounded-[28px] border border-line bg-line sm:grid-cols-2 lg:grid-cols-4">
        {reasons.map((r) => (
          <li key={r.title} className="bg-surface p-7 transition-colors hover:bg-surface-raised">
            <h3 className="flex flex-wrap items-center gap-2 font-semibold tracking-tight">
              {r.title}
              {r.soon && (
                <span className="rounded-full bg-suds px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.08em] text-on-suds">
                  Coming soon
                </span>
              )}
            </h3>
            <p className="mt-2 text-sm leading-relaxed text-ink-muted">{r.body}</p>
          </li>
        ))}
      </Reveal>
    </section>
  );
}
