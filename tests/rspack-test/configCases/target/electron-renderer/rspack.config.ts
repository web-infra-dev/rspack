import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'electron-renderer',
  optimization: {
    minimize: false,
  },
});
