import { defineConfig } from '@rspack/cli';

import { rspack as webpack } from '@rspack/core';

export default defineConfig({
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
});
