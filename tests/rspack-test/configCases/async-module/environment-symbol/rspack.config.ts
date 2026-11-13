import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default [true, false].map((symbol, i) =>
  defineConfig({
    output: {
      filename: `bundle${i}.js`,
      environment: {
        symbol,
      },
    },
    plugins: [
      new rspack.DefinePlugin({
        SUPPORTS_SYMBOL: JSON.stringify(symbol),
      }),
    ],
  }),
);
