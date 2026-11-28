import { defineConfig } from '@rspack/cli';

export default defineConfig(
  (['source-map', 'cheap-module-source-map'] as const).map((devtool) => ({
    mode: 'development',
    devtool,
    node: { __dirname: false, __filename: false },
    module: {
      rules: [{ test: /module\.js$/, use: ['./loader.mjs'] }],
    },
  })),
);
