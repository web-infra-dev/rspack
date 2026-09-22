import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.js$/,
        type: 'javascript/esm',
      },
    ],
  },
});
