import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.normalModuleFactory.tap('getResolver', (nmf) => {
          const resolver = nmf.getResolver('normal', {});
          expect(resolver).toBeTruthy();
        });
      },
    }),
  ],
});
