import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        test: /\.css/,
        parser: {
          namedExports: false,
        },
        type: 'css/module',
      },
    ],
  },
});
