// Clackboard's mark: a keycap with two sound arcs on a graphite tile (the app icon, small).
// Fixed colours, so it reads the same in light and dark themes.
export function markSvg(className = "hb-mark"): string {
  return `<svg class="${className}" viewBox="80 80 864 864" aria-hidden="true" focusable="false">
    <rect x="80" y="80" width="864" height="864" rx="196" fill="#111518"/>
    <g transform="translate(113 104) scale(6)">
      <rect x="10" y="30" width="78" height="76" rx="20" fill="#139a71"/>
      <rect x="21" y="34" width="56" height="52" rx="13" fill="#2ee6a8"/>
      <g fill="none" stroke="#2ee6a8" stroke-linecap="round" stroke-width="9">
        <path d="M98 52a20 20 0 0 1 0 32"/>
        <path d="M108 40a34 34 0 0 1 0 56"/>
      </g>
    </g>
  </svg>`;
}
