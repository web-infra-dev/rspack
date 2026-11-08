import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  performance: {
    hints: false,
  },
});
