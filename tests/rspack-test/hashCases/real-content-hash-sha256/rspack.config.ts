import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './src/index.js',
  devtool: false,
  output: {
    filename: 'main.js',
    hashFunction: 'sha256',
    hashDigestLength: 64,
    assetModuleFilename: '[contenthash][ext]',
  },
  module: {
    rules: [
      {
        test: /\.(png|jpg|svg)$/,
        type: 'asset/resource',
      },
    ],
  },
  context: import.meta.dirname,
  optimization: {
    realContentHash: true,
  },
});
