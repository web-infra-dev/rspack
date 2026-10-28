import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.svg$/,
        type: 'asset/resource',
      },
    ],
  },
  optimization: {
    sideEffects: true,
  },
  externalsPresets: {
    node: true,
  },
});
