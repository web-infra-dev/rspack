import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  entry: './index.mjs',
  performance: {
    hints: false,
  },
  optimization: {
    minimize: false,
  },
});
