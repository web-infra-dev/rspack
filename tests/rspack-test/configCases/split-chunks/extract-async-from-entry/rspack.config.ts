import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index',
    second: './index',
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      minSize: 1,
    },
  },
});
