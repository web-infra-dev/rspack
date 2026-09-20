import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.svg$/,
        type: 'asset/resource',
      },
      {
        test: /\.css/,
        type: 'css/auto',
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
