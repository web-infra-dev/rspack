import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index.js',
  },
  devtool: false,
  externalsPresets: {
    node: true,
  },
  node: {
    __dirname: false,
  },
});
