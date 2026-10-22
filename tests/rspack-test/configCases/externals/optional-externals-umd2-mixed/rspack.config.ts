import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: { type: 'umd2' },
  },
  externals: {
    external: 'external',
    external2: 'fs',
  },
});
