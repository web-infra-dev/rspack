import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  cache: true,
  output: {
    pathinfo: true,
  },
  optimization: {
    minimize: false,
    concatenateModules: false,
  },
});
