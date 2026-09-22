import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: { type: 'umd' },
  },
  externals: {
    external: 'external',
  },
});
