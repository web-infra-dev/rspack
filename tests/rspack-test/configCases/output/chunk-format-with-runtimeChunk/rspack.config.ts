import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: {
    main: './index.js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
  output: {
    filename: '[name].mjs',
    module: true,
    chunkFormat: 'module',
  },
});
