import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'fs',
    os: 'os',
    path: 'path',
  },
});
