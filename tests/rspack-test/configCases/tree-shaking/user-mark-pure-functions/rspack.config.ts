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
        test: /decl\.js/,
        parser: {
          pureFunctions: ['pureFn', 'notExistFunction'],
        },
      },
    ],
  },
});
