import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.png$/,
        use: [{ loader: 'file-loader', options: { esModule: false } }],
        type: 'javascript/auto',
      },
    ],
  },
});
