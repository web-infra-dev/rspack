import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    module: true,
    library: {
      type: 'module',
    },
  },
  optimization: {
    concatenateModules: true,
  },
});
