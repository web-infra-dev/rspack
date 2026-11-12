import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externalsType: 'var',
  externals: {
    external: 'external',
  },
});
