import { defineConfig } from '@rspack/cli';

export default defineConfig({
  experiments: {
    runtimeMode: 'rspack',
  },
  output: {
    filename: 'main.js',
  },
  optimization: {
    concatenateModules: false,
    usedExports: false,
  },
});
