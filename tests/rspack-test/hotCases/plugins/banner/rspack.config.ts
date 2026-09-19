import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.BannerPlugin({
      banner:
        "globalThis.bannerIndex = typeof globalThis.bannerIndex === 'number' ? globalThis.bannerIndex + 1 : 0;",
      raw: true,
    }),
  ],
});
