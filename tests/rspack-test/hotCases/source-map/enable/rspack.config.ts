import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index.js',
  },
  devtool: 'cheap-source-map',
  externalsPresets: {
    node: true,
  },
  node: {
    __dirname: false,
  },
});
