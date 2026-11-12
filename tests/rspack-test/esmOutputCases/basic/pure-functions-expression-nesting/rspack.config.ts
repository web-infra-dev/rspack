import { defineConfig } from '@rspack/cli';

export default defineConfig({
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
        test: /decl\.js$/,
        parser: {
          pureFunctions: ['a', 'b'],
        },
      },
    ],
  },
});
