// @ts-check
import { defineConfig } from 'astro/config';

import react from '@astrojs/react';
import partytown from '@astrojs/partytown';
import sitemap from '@astrojs/sitemap';

// https://astro.build/config
export default defineConfig({
  // Update when the production domain is finalized — sitemap + canonical
  // URLs derive from it.
  site: 'https://cleft.app',
  integrations: [
    react(),
    // Partytown runs third-party scripts (analytics, etc.) off the main
    // thread. Add scripts with type="text/partytown" in Layout.astro.
    partytown({ config: { forward: ['dataLayer.push'] } }),
    sitemap(),
  ],
});
