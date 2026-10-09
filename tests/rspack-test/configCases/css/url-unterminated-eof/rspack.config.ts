import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: {
    __dirname: false,
  },
  mode: 'development',
  module: {
    rules: [
      {
        test: /style\.css$/,
        type: 'css',
        use: ['./emit-css.mjs'],
      },
    ],
  },
});
