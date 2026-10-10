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
        let rejected: JsLoaderContext;
        plugin.options = async (context: JsLoaderContext) => {
          const result = await run(context);
          if (context.loaderState === binding.JsLoaderState.Normal) {
            rejected = context;
            throw new Error('loader runner rejected after executing');
          }
          return result;
        };
        compiler.hooks.afterCompile.tap('LoaderContextRejection', () => {
          assert.ok(rejected);
          assert.equal(Object.hasOwn(rejected, 'loaderItems'), false);
          assert.throws(
            () => rejected.loaderItems,
            /outside the active JavaScript loader invocation/,
          );
        });
      },
    }),
  ],
});
