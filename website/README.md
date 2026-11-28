# Clackboard website

The landing page for Clackboard, built with Next.js and Tailwind CSS as a static site. It started from Hushboard's landing page (MIT, by Tekcify) and was rebuilt around the sounds: the MacBook in the hero is playable, using the same switch recordings as the app.

```bash
npm install
npm run dev      # http://localhost:3000
npm run build    # static files in out/
```

- Colours follow the app's design system in `../design-system/project/tokens.json` (graphite with an electric mint accent). The logo is `public/brand.svg`, also the favicon (`src/app/icon.svg`).
- The demo's sounds live in `public/packs/`, made from the app's packs with `cargo run -p site-packs --release -- src-tauri/resources/packs/keyboard website/public/packs <pack id>…` from the repo root. The page plays them with `src/lib/sound.ts`.
- Links, the current version and the switch list are in `src/content/site.ts`.

## Deploying

- **GitHub Pages**: `.github/workflows/pages.yml` builds with `NEXT_PUBLIC_BASE_PATH=/clackboard` and publishes `out/` on every push to `main` that touches `website/`.
- **Cloudflare Pages**: root directory `website`, build command `npm run build`, output directory `out`. Leave `NEXT_PUBLIC_BASE_PATH` unset. `public/_headers` sets cache headers there.
