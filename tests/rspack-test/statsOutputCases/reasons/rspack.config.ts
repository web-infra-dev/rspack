import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index',
  stats: {
    all: false,
    modules: true,
    reasons: true,
  },
});
