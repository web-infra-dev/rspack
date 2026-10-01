import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: {
      type: 'umd',
      root: ['test', 'library'],
      amd: 'test-library',
      commonjs: 'test-library',
    },
  },
});
