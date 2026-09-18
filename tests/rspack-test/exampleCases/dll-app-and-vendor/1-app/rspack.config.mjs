import path from 'node:path';
import { rspack as webpack } from '@rspack/core';
import { readFileSync } from 'node:fs';
export default {
  // mode: "development" || "production",
  context: import.meta.dirname,
  entry: './example-app',
  output: {
    filename: 'app.js',
    path: path.resolve(import.meta.dirname, 'dist'),
  },
  plugins: [
    new webpack.DllReferencePlugin({
      manifest: JSON.parse(
        readFileSync(
          new URL('../0-vendor/dist/vendor-manifest.json', import.meta.url),
          'utf-8',
        ),
      ), // eslint-disable-line
    }),
  ],
};
