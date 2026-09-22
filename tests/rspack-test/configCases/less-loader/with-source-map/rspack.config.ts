import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  devtool: 'source-map',
  output: {
    publicPath: '/',
  },
  module: {
    rules: [
      {
        test: /\.less$/,
        use: [{ loader: 'less-loader' }],
        type: 'css',
        generator: {
          exportsOnly: false,
        },
      },
      {
        resourceQuery: /resource/,
        type: 'asset/resource',
        generator: {
          filename: 'source.txt',
        },
      },
    ],
  },
});
