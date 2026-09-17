import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new rspack.BannerPlugin({
      banner:
        "globalThis.bannerIndex = typeof globalThis.bannerIndex === 'number' ? globalThis.bannerIndex + 1 : 0;",
      raw: true,
    }),
  ],
};
