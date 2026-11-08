import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  entry: {
    main: './index.js',
    worker: './lib.js',
  },
  optimization: {
    runtimeChunk: false,
  },
});
