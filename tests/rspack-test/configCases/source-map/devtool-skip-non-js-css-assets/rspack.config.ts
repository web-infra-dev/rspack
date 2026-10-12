import { defineConfig } from '@rspack/cli';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.gltf$/,
        type: 'asset/resource',
        generator: {
          filename: 'text/[name][ext]',
          binary: false,
        },
      },
    ],
  },
});
