import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  output: {
    bundlerInfo: {
      force: true,
    },
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.compilation.tap('test', (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'test',
          () => {},
        );
      });
    }),
  ],
});
