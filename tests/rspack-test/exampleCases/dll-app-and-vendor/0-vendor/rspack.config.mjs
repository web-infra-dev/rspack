import path from 'node:path';
import { rspack as webpack } from '@rspack/core';
export default {
  // mode: "development" || "production",
  context: import.meta.dirname,
  entry: ['example-vendor'],
  output: {
    filename: 'vendor.js', // best use [fullhash] here too
    path: path.resolve(import.meta.dirname, 'dist'),
    library: 'vendor_lib_[fullhash]',
  },
  plugins: [
    new webpack.DllPlugin({
      name: 'vendor_lib_[fullhash]',
      path: path.resolve(import.meta.dirname, 'dist/vendor-manifest.json'),
    }),
  ],
};
