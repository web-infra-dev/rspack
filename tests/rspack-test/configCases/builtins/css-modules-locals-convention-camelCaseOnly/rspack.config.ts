import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    path: 'node-commonjs path',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
        generator: {
          exportsConvention: 'camel-case-only',
        },
      },
    ],
  },
});
