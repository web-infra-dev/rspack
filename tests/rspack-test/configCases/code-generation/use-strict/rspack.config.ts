import { defineConfig } from '@rspack/cli';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  optimization: {
    concatenateModules: false,
    minimize: false,
  },
});
