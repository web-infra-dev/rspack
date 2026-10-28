import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  plugins: [
    definePlugin(function (compiler) {
      compiler.hooks.contextModuleFactory.tap(
        'test',
        (contextModuleFactory) => {
          contextModuleFactory.hooks.afterResolve.tap('test', () => {
            // do nothing
          });
        },
      );
    }),
  ],
});
