import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: {
      type: 'umd',
      root: 'testLibrary[name]',
      amd: 'test-library',
      commonjs: 'test-library-[name]',
    },
  },
  externals: 'module',
});
