import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index',
  },
  target: 'node',
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: false,
  },
});
