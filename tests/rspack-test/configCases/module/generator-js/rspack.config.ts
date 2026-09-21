import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  output: {
    publicPath: '/',
    filename: 'main.js',
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        type: 'javascript/auto',
        generator: {
          filename: 'custom/lib.js',
        },
      },
    ],
  },
});
