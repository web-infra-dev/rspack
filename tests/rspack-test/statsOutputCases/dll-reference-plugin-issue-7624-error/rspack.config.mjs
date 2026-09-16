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
      manifest: import.meta.dirname + '/blank-manifest.json',
      name: 'blank-manifest',
    }),
  ],
};
