import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';
import binding, { type JsLoaderContext } from '@rspack/binding';

export default defineConfig({
  context: import.meta.dirname,
  module: { rules: [{ test: /fixture\.js$/, use: './loader.mjs' }] },
  plugins: [
    definePlugin({
      apply(compiler) {
        const plugin = compiler.__internal__builtinPlugins.find(
          (plugin) =>
            plugin.name === binding.BuiltinPluginName.JsLoaderRspackPlugin,
        )!;
        const run = plugin.options as (
          context: JsLoaderContext,
        ) => Promise<JsLoaderContext>;
        let invalid: JsLoaderContext;
        plugin.options = async (context: JsLoaderContext) => {
          const result = await run(context);
          if (context.loaderState === binding.JsLoaderState.Normal) {
            invalid = context;
            Object.defineProperty(result, 'cacheable', {
              get() {
                throw new Error('loader context result conversion failed');
              },
              configurable: true,
            });
          }
          return result;
        };
        compiler.hooks.afterCompile.tap('LoaderContextConversionError', () => {
          assert.ok(invalid);
          assert.equal(Object.hasOwn(invalid, 'loaderItems'), false);
          assert.throws(
            () => invalid.loaderItems,
            /outside the active JavaScript loader invocation/,
          );
        });
      },
    }),
  ],
});
