import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import { rspack as webpack } from '@rspack/core';

export default defineConfig({
  entry: ['./index'],
  output: {
    filename: 'dll.js',
    chunkFilename: '[id].dll.js',
    library: { type: 'commonjs2' },
  },
  module: {
    rules: [
      {
        test: /0-create-dll.(module|dependency)/,
        sideEffects: false,
      },
    ],
  },
  optimization: {
    usedExports: true,
    sideEffects: true,
  },
  plugins: [
    new webpack.DllPlugin({
      path: path.resolve(
        import.meta.dirname,
        '../../../js/config/dll-plugin-side-effects/manifest0.json',
      ),
      entryOnly: false,
    }),
  ],
});
