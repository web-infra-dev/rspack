import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    uniqueName: 'app',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
