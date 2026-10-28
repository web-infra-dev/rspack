import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  externals: {
    fs: 'fs',
    os: 'os',
    path: 'path',
  },
});
