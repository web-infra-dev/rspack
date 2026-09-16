import { SwcJsMinimizerRspackPlugin } from '@rspack/core';

/** @type {import("@rspack/coresrc/index").RspackOptions} */
export default {
  plugins: [
    new SwcJsMinimizerRspackPlugin({
      minimizerOptions: {
        format: {
          asciiOnly: true,
        },
      },
    }),
  ],
};
