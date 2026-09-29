import { defineConfig } from '@rspack/cli';

export default defineConfig({
  resolve: {
    extensions: ['.ts', '...'],
  },
  optimization: {
    minimize: false,
    moduleIds: 'named',
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        type: 'javascript/auto',
      },
    ],
  },
});
