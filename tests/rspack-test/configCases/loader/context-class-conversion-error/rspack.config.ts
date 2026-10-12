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
          // Conversion failure can leave JS-owned snapshots on the result.
          // Accessing the native getter must fail after the invocation ends.
          const getter = Object.getOwnPropertyDescriptor(
            Object.getPrototypeOf(invalid),
            'loaderItems',
          )!.get!;
          assert.throws(
            () => getter.call(invalid),
            /outside the active JavaScript loader invocation/,
          );
        });
      },
    }),
  ],
});
