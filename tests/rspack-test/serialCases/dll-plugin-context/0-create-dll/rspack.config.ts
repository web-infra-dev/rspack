import { defineConfig } from '@rspack/cli';

import path from 'node:path';
import { rspack as webpack } from '@rspack/core';

export default defineConfig({
  entry: ['.'],
  output: {
    filename: 'dll.js',
    chunkFilename: '[id].dll.js',
    library: { type: 'commonjs2' },
  },
  plugins: [
    new webpack.DllPlugin({
      path: path.resolve(
        import.meta.dirname,
        '../../../js/config/dll-plugin-context/manifest0.json',
      ),
      // Context modules only land in the manifest when entryOnly is false.
      entryOnly: false,
    }),
  ],
});
