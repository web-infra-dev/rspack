import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    module: true,
    library: {
      type: 'module',
    },
  },
  optimization: {
    usedExports: true,
    concatenateModules: true,
  },
  target: 'browserslist: last 2 chrome versions',
});
