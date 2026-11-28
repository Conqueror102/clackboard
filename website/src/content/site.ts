// One place for links and the current release, so a new version is a one-line change.
export const version = "0.2.0";

const repo = "https://github.com/Conqueror102/clackboard";
const release = `${repo}/releases/download/v${version}`;

/** The site may live under a sub-path (GitHub Pages serves it at /clackboard). */
export const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";
/** A file from `public/`, with the sub-path in front. */
export const asset = (path: string) => `${basePath}${path}`;

export const links = {
  repo,
  releases: `${repo}/releases/latest`,
  changelog: `${repo}/blob/main/CHANGELOG.md`,
  installGuide: `${repo}/blob/main/docs/installing.md`,
  contributing: `${repo}/blob/main/CONTRIBUTING.md`,
  security: `${repo}/blob/main/SECURITY.md`,
  license: `${repo}/blob/main/LICENSE`,
  credits: `${repo}/blob/main/CREDITS.md`,
  buildFromSource: `${repo}#build-from-source`,
  hushboard: "https://github.com/tekcify/hushboard",
  mechvibes: "https://github.com/hainguyents13/mechvibes",
};

export type Platform = "mac" | "windows" | "linux";

export const downloads: Record<Platform, { label: string; href: string }> = {
  mac: { label: "Download for macOS", href: `${release}/Clackboard_${version}_universal.dmg` },
  windows: { label: "Download for Windows", href: `${release}/Clackboard_${version}_x64-setup.exe` },
  linux: { label: "Download for Linux", href: `${release}/Clackboard_${version}_amd64.deb` },
};

// Every build, for the "other downloads" menu. File names follow docs/installing.md.
export const allDownloads: { platform: Platform; name: string; note: string; href: string }[] = [
  { platform: "mac", name: "macOS", note: ".dmg · Apple silicon and Intel", href: downloads.mac.href },
  { platform: "windows", name: "Windows", note: ".exe · 64-bit installer", href: downloads.windows.href },
  { platform: "linux", name: "Linux · Debian, Ubuntu", note: ".deb · x86-64", href: `${release}/Clackboard_${version}_amd64.deb` },
  { platform: "linux", name: "Linux · Fedora, openSUSE", note: ".rpm · x86-64", href: `${release}/Clackboard-${version}-1.x86_64.rpm` },
  { platform: "linux", name: "Linux · any distribution", note: "AppImage · x86-64", href: `${release}/Clackboard_${version}_amd64.AppImage` },
];

export type Kind = "clicky" | "tactile" | "linear" | "topre";

/** Every switch in the app, grouped for the switches section. `demo` = playable on this page. */
export const switches: { name: string; kind: Kind; note: string; demo?: string }[] = [
  { name: "Cherry MX Blue", kind: "clicky", note: "ABS · PBT · kbsim", demo: "cherrymx-blue-abs" },
  { name: "IBM Model M", kind: "clicky", note: "Buckling spring, every key recorded", demo: "ibm-model-m" },
  { name: "Typewriter", kind: "clicky", note: "Antique", demo: "typewriter" },
  { name: "Holy Panda", kind: "tactile", note: "kbsim", demo: "holy-panda" },
  { name: "EG Oreo", kind: "tactile", note: "Classic · New", demo: "eg-oreo" },
  { name: "Cherry MX Brown", kind: "tactile", note: "ABS · PBT · kbsim" },
  { name: "EG Crystal Purple", kind: "tactile", note: "Crisp" },
  { name: "Logitech G915 TKL", kind: "tactile", note: "Low-profile brown" },
  { name: "NovelKeys Cream", kind: "linear", note: "Self-lubricating", demo: "nk-cream" },
  { name: "Cherry MX Red", kind: "linear", note: "ABS", demo: "cherrymx-red-abs" },
  { name: "Cherry MX Black", kind: "linear", note: "ABS · PBT · kbsim" },
  { name: "Cherry MX Speed Silver", kind: "linear", note: "Short travel" },
  { name: "Gateron Ink", kind: "linear", note: "Black · Red" },
  { name: "Telios V2", kind: "linear", note: "Smooth" },
  { name: "Banana Split", kind: "linear", note: "Deep" },
  { name: "Alpaca", kind: "linear", note: "kbsim" },
  { name: "SteelSeries Apex Pro", kind: "linear", note: "Magnetic" },
  { name: "Topre Purple Hybrid", kind: "topre", note: "PBT", demo: "topre-purple-hybrid-pbt" },
];

export const kindLabel: Record<Kind, string> = { clicky: "Clicky", tactile: "Tactile", linear: "Linear", topre: "Topre" };
