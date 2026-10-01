import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  entry: 'data:text/javascript,import "./index.js";',
  plugins: [
    definePlugin(function (compiler) {
      compiler.hooks.compilation.tap(
        'test',
        (_compilation, { normalModuleFactory }) => {
          normalModuleFactory.hooks.afterResolve.tap('test', () => {});
        },
      );
    }),
  ],
});
