import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  target: 'node22',
  cache: false,
  module: {
    rules: [
      {
        test: /declaration-(?:runtime-)?(object|array|default)\.js$/,
        type: 'javascript/auto',
      },
    ],
  },
  output: {
    filename: 'bundle0.mjs',
    library: { type: 'modern-module' },
  },
  optimization: { minimize: false, runtimeChunk: false },
});
