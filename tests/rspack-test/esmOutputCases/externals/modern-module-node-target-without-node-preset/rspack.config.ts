import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externalsType: 'modern-module',
  externalsPresets: {
    node: false,
  },
  externals: {
    fs: 'fs',
    os: 'os',
    path: 'path',
  },
});
