import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index',
  },
  devtool: 'source-map',
  target: 'node',
  output: {
    filename: '[name].js',
    sourceMapFilename: '../maps/[name].js.map',
  },
});
