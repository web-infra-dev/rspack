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
        test: /dep-named\.js$/,
        parser: {
          pureFunctions: ['pureNamed'],
        },
      },
      {
        test: /dep-default\.js$/,
        parser: {
          pureFunctions: ['default'],
        },
      },
    ],
  },
});
