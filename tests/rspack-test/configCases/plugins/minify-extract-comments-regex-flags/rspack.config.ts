import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  optimization: {
    minimize: true,
    minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin({
        extractComments: {
          // This regex uses negative lookahead (?! ...) which is supported by regress but not by Rust's regex crate
          // It should extract comments starting with /*! or /** but NOT those containing specific keywords
          condition:
            /^\**!(?! *(SuppressStringValidation|StartNoStringValidationRegion|EndNoStringValidationRegion))/i,
        },
      }),
    ],
  },
});
