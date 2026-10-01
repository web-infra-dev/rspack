import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  mode: 'development',
  devtool: 'source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
