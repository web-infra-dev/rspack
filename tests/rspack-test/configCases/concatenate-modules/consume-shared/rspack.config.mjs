import { rspack } from '@rspack/core';

/**@type {import("@rspack/core").Configuration}*/
export default {
  optimization: {
    concatenateModules: true,
  },
  plugins: [
    new rspack.sharing.ConsumeSharedPlugin({
      consumes: {
        './lib/c.js': {
          singleton: true,
          eager: true,
        },
      },
    }),
  ],
};
