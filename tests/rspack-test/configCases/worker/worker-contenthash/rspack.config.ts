import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: {
      import: './index.js',
      filename: '[name].js',
    },
  },
  output: {
    filename: '[name]-[contenthash].js',
  },
});
