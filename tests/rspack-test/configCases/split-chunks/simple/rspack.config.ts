import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    vendor: ['./a'],
    main: './index',
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      minSize: 1,
      name: 'vendor',
    },
  },
});
