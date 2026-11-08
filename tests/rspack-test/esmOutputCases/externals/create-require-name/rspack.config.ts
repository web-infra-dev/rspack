import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externalsType: 'modern-module',
  externals: {
    'node:module': 'node:module',
  },
});
