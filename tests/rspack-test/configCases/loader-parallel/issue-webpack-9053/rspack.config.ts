import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /c\.js$/,
        use: [{ loader: 'loader2', parallel: true, options: {} }],
      },
      {
        test: /d\.js$/,
        use: [{ loader: 'loader3', parallel: true, options: {} }],
      },
    ],
  },
});
