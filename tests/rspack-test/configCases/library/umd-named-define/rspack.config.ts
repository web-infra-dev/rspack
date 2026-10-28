import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: {
      type: 'umd',
      root: 'testLibrary[name]',
      amd: 'test-library-[name]',
      commonjs: 'test-library-[name]',
      umdNamedDefine: true,
    },
  },
  externals: 'module',
});
