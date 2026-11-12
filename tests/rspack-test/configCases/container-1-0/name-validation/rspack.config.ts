import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    '@rspack/core': 'node-commonjs @rspack/core',
  },
});
