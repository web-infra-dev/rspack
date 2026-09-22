import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index',
  stats: 'errors-warnings',
  module: {
    rules: [
      {
        test: /.js/,
        type: 'javascript/esm',
      },
    ],
  },
});
