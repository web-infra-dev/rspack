import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: {
      type: 'umd',
      name: 'NamedLibrary',
      umdNamedDefine: true,
      auxiliaryComment: {
        commonjs: 'test comment commonjs',
        commonjs2: 'test comment commonjs2',
        amd: 'test comment amd',
        root: 'test comment root',
      },
    },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
});
