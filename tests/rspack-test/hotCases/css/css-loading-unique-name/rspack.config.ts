import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    uniqueName: 'css-test',
  },
  module: {
    rules: [
      {
        test: /\.css/,
        type: 'css/auto',
      },
    ],
  },
});
