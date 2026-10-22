import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'module-import fs',
    'node:fs': 'module-import fs',
  },
});
