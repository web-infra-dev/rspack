import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: { type: 'commonjs2' },
  },
  externals: {
    external: 'external',
  },
});
