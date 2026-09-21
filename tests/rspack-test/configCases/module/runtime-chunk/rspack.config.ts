import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    module: true,
    filename: '[name].mjs',
  },
  target: ['web', 'es2020'],
  optimization: {
    minimize: true,
    runtimeChunk: 'single',
  },
});
