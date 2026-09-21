import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    path: 'node-commonjs path',
  },
  target: 'web',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/i,
        type: 'css',
      },
    ],
  },
});
