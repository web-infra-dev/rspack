import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  optimization: {
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    concatenateModules: false,
  },
  experiments: {
    pureFunctions: true,
  },
  module: {
    rules: [
      {
        test: /dep\.js$/,
        parser: {
          pureFunctions: ['fromObject', 'fromArray'],
        },
      },
    ],
  },
});
