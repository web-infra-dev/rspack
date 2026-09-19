import { defineConfig } from '@rspack/cli';

import path from 'node:path';
import { rspack as webpack } from '@rspack/core';
import { readFileSync } from 'node:fs';
export default defineConfig({
  // mode: "development" || "production",
  plugins: [
    new webpack.DllReferencePlugin({
      context: path.join(import.meta.dirname, '..', 'dll'),
      manifest: JSON.parse(
        readFileSync(
          new URL('../dll/dist/alpha-manifest.json', import.meta.url),
          'utf-8',
        ),
      ), // eslint-disable-line
    }),
    new webpack.DllReferencePlugin({
      scope: 'beta',
      manifest: JSON.parse(
        readFileSync(
          new URL('../dll/dist/beta-manifest.json', import.meta.url),
          'utf-8',
        ),
      ), // eslint-disable-line
      extensions: ['.js', '.jsx'],
    }),
  ],
});
