import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  devtool: false,
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
});
