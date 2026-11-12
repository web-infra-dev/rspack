import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
        generator: {
          exportsOnly: false,
        },
      },
    ],
  },
});
