/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './src/index.js',
  devtool: false,
  output: {
    filename: 'main.js',
    hashFunction: 'md4',
    hashDigestLength: 20,
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
};
