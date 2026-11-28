import type { NextConfig } from "next";

// A static site: `next build` writes plain files to `out/`, which GitHub Pages and Cloudflare
// Pages serve as they are. GitHub Pages hosts it under /clackboard, so the build for it sets
// NEXT_PUBLIC_BASE_PATH=/clackboard; at a domain's root it stays empty.
const basePath = process.env.NEXT_PUBLIC_BASE_PATH || undefined;

const nextConfig: NextConfig = {
  output: "export",
  basePath,
  trailingSlash: true,
  images: { unoptimized: true },
  turbopack: {
    root: __dirname,
    rules: {
      "*.css": {
        loaders: ["@tailwindcss/turbopack"],
        as: "*.css",
      },
    },
  },
};

export default nextConfig;
