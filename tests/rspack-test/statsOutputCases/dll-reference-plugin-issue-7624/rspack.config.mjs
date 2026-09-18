import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './entry.js',
  output: {
    filename: 'bundle.js',
  },
  plugins: [
    new webpack.DllReferencePlugin({
      manifest: import.meta.dirname + '/non-blank-manifest.json',
      name: 'non-blank-manifest',
    }),
  ],
};
