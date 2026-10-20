import { defineConfig } from '@rspack/cli';

export default defineConfig({
  amd: {},
  optimization: {
    concatenateModules: false,
  },
});
