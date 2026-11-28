import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  mode: 'development',
  devtool: false,
  // Two async entry modules, so the one the bundle exports and the one it does not
  // are both awaited behind the chunk the split carries out of them.
  entry: { main: ['./a.js', './b.js'] },
  output: {
    module: true,
    filename: '[name].mjs',
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        shared: {
          test: /shared\.js$/,
          name: 'shared',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
});
