import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  module: {
    parser: {
      json: {
        exportsDepth: Number.MAX_SAFE_INTEGER,
      },
    },
  },
});
