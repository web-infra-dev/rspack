import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    'virtual-fs': 'module fs',
  },
});
