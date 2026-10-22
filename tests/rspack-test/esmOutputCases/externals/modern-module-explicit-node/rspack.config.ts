import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externalsType: 'modern-module',
  externals: {
    fs: 'fs',
    os: 'os',
    path: 'path',
  },
});
