import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: {
      import: './index',
      runtime: false,
    },
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
});
