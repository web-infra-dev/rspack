import path from 'node:path';
import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: ['.'],
  resolve: {
    extensions: ['.js'],
  },
  output: {
    filename: 'dll.js',
    chunkFilename: '[id].dll.js',
    library: { type: 'commonjs2' },
  },
  plugins: [
    new webpack.DllPlugin({
      path: path.resolve(
        import.meta.dirname,
        '../../../js/config/dll-plugin-format/manifest0.json',
      ),
      format: true,
    }),
  ],
};
