import Image from "next/image";
import { links, asset } from "@/content/site";
import { Reveal } from "./Reveal";

const columns = [
  {
    title: "Product",
    items: [
      { label: "How it works", href: "#how" },
      { label: "Switches", href: "#switches" },
      { label: "Download", href: "#download" },
      { label: "FAQ", href: "#faq" },
      { label: "What's new", href: links.changelog },
    ],
  },
  {
    title: "Help",
    items: [
      { label: "Install guide", href: links.installGuide },
      { label: "Report a bug", href: `${links.repo}/issues` },
      { label: "Security", href: links.security },
    ],
  },
  {
    title: "Project",
    items: [
      { label: "GitHub", href: links.repo },
      { label: "Contributing", href: links.contributing },
      { label: "Build from source", href: links.buildFromSource },
      { label: "MIT license", href: links.license },
      { label: "Credits", href: links.credits },
    ],
  },
];

export function Footer() {
  return (
    <footer className="border-t border-line">
      <div className="mx-auto grid max-w-6xl gap-12 px-5 py-14 md:grid-cols-[1.4fr_repeat(3,1fr)]">
        <Reveal variant="soft">
          <a href="#top" className="inline-flex items-center gap-2.5 font-semibold tracking-tight">
            <Image src={asset("/brand.svg")} alt="" width={30} height={30} />
            Clackboard
          </a>
          <p className="mt-4 max-w-56 text-sm leading-relaxed text-ink-muted">
            A real switch sound for every key you press, on any keyboard.
          </p>
        </Reveal>
        {columns.map((c, i) => (
          <Reveal as="nav" variant="soft" delay={(i + 1) * 90} key={c.title} aria-label={c.title}>
            <h3 className="text-sm font-semibold">{c.title}</h3>
            <ul className="mt-4 space-y-2.5 text-sm text-ink-muted">
              {c.items.map((i) => (
                <li key={i.label}>
                  <a className="transition-colors hover:text-ink" href={i.href}>
                    {i.label}
                  </a>
                </li>
              ))}
            </ul>
          </Reveal>
        ))}
      </div>
      <div className="mx-auto flex max-w-6xl flex-wrap justify-between gap-3 border-t border-line px-5 py-6 text-xs text-ink-muted">
        <span>© 2026 the Clackboard contributors · Built on Hushboard by Tekcify (MIT)</span>
        <span>Switch sounds from Mechvibes, kbsim, Keyboard Sounds and bucklespring</span>
      </div>
    </footer>
  );
}
