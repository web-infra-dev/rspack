import fs from 'node:fs';
import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: ['css-loader'],
      },
    ],
  },
  experiments: {
    css: false,
    useInputFileSystem: [/.*/],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.inputFileSystem = fs;
      },
    }),
  ],
});
