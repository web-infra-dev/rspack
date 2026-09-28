import { defineConfig } from '@rspack/cli';

export default defineConfig({
  incremental: true,
  entry: {
    first: {
      import: './first.js',
      filename: '[name].[fullhash].js',
    },
    second: './second.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
});
