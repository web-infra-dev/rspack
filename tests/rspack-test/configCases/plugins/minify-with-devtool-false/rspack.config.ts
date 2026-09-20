import { defineConfig } from '@rspack/cli';

import { SourceMapDevToolPlugin } from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,
  target: 'node',
  entry: {
    main: './index.js',
  },
  optimization: {
    minimize: true,
  },
  plugins: [new SourceMapDevToolPlugin({})],
  devtool: false,
});
