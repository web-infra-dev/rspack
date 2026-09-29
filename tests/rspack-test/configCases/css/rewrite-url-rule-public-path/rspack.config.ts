import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  output: {
    publicPath: 'auto',
  },
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.png$/i,
        type: 'asset/resource',
        generator: {
          filename: '[name][ext]',
          publicPath: 'https://test.rspack.rs/cdn/',
        },
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
