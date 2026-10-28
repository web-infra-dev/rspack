import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  externalsType: 'modern-module',
  externals: {
    fs: 'fs',
    os: 'os',
    path: 'path',
  },
});
