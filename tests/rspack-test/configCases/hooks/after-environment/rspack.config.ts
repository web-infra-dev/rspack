import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.afterEnvironment.tap('getResolver', () => {
          expect(compiler.resolverFactory).toBeTruthy();
        });
      },
    }),
  ],
});
