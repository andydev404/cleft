// @ts-check
import { defineConfig } from 'astro/config';

import react from '@astrojs/react';
import partytown from '@astrojs/partytown';
import sitemap from '@astrojs/sitemap';

// https://astro.build/config
export default defineConfig({
  // GitHub Pages for the andydev404/cleft repo. When a custom domain
  // lands, set `site` to it and drop `base` — canonical URLs, the sitemap,
  // and robots.txt (public/robots.txt) all derive from these.
  site: 'https://andydev404.github.io',
  base: '/cleft',
  integrations: [
    react(),
    // Partytown runs third-party scripts (analytics, etc.) off the main
    // thread. Add scripts with type="text/partytown" in Layout.astro.
    partytown({ config: { forward: ['dataLayer.push'] } }),
    sitemap(),
  ],
});
