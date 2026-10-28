import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: {
      type: 'umd',
      root: 'testLibrary',
      amd: 'test-library',
      commonjs: 'test-library',
    },
  },
});
