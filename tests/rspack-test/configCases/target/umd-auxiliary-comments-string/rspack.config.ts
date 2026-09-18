import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: {
      type: 'umd',
      name: 'NamedLibrary',
      umdNamedDefine: true,
      auxiliaryComment: 'test comment',
    },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
});
