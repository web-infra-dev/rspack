import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index',
    misc: './second',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      minSize: 0,
    },
  },
});
