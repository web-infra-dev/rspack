import { defineConfig } from '@rspack/cli';

import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  output: {
    environment: {
      bigIntLiteral: true,
    },
  },
  plugins: [
    new DefinePlugin({
      BIGINT: BigInt('9007199254740993'),
      ZERO_BIGINT: BigInt(0),
    }),
  ],
});
