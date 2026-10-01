import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  devtool: false,
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
});
