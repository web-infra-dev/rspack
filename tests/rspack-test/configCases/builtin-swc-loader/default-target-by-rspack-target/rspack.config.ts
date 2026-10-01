import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  target: ['async-node', 'browserslist:node > 16'],
  module: {
    rules: [
      {
        test: /\.js$/,
        use: 'builtin:swc-loader',
      },
    ],
  },
});
