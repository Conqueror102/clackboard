"use client";

import { useEffect, useState } from "react";
import { links } from "@/content/site";

const format = (n: number) => (n >= 1000 ? `${(n / 1000).toFixed(n >= 10000 ? 0 : 1)}k` : String(n));

// A pill for the header: "Star" with the GitHub mark, a divider, then the count. The site is
// static, so the count is fetched in the browser; without it the pill just says "Star".
export function StarButton() {
  const [stars, setStars] = useState<number | null>(null);

  useEffect(() => {
    const ctrl = new AbortController();
    fetch("https://api.github.com/repos/Conqueror102/clackboard", {
      headers: { Accept: "application/vnd.github+json" },
      signal: ctrl.signal,
    })
      .then((r) => (r.ok ? r.json() : null))
      .then((d: { stargazers_count?: number } | null) => {
        // A brand-new repo shows plain "Star" rather than a lonely 0.
        if (typeof d?.stargazers_count === "number" && d.stargazers_count > 0) setStars(d.stargazers_count);
      })
      .catch(() => {});
    return () => ctrl.abort();
  }, []);

  return (
    <a
      href={links.repo}
      className="group inline-flex h-9 items-center rounded-full border border-line-strong/60 bg-surface-raised/60 text-sm font-semibold backdrop-blur-xl transition-colors hover:border-line-strong"
      aria-label={stars == null ? "Star Clackboard on GitHub" : `Star Clackboard on GitHub, ${stars} stars`}
    >
      <span className="flex h-full items-center gap-2 pl-4 pr-3.5">
        <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden fill="currentColor" className="text-ink-muted transition-colors group-hover:text-ink">
          <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
        </svg>
        Star
      </span>
      {stars != null && (
        <span className="flex h-full items-center border-l border-line-strong/60 pl-3.5 pr-4 tabular-nums">{format(stars)}</span>
      )}
    </a>
  );
}
