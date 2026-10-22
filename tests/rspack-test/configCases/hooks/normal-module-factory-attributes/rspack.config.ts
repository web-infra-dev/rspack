import { defineConfig, definePlugin } from '@rspack/cli';
import { type ResolveData } from '@rspack/core';

const pluginName = 'attributes-plugin';

export default defineConfig({
  plugins: [
    definePlugin({
      apply(compiler) {
        const seen: Record<string, Record<string, unknown>> = {};

        const record = (hook: string, resolveData: ResolveData) => {
          const perHook = (seen[resolveData.request] ??= {});
          perHook[hook] =
            resolveData.attributes === undefined
              ? '<undefined>'
              : resolveData.attributes === null
                ? '<null>'
                : resolveData.attributes;
        };

        compiler.hooks.compilation.tap(
          pluginName,
          (compilation, { normalModuleFactory }) => {
            for (const hook of [
              'beforeResolve',
              'factorize',
              'resolve',
              'afterResolve',
            ] as const) {
              normalModuleFactory.hooks[hook].tap(pluginName, (resolveData) => {
                record(hook, resolveData);
                // `attributes` is read-only; this rewrite must not reach the
                // later hooks, and must not reach the module rules either.
                if (hook === 'beforeResolve' && resolveData.attributes) {
                  resolveData.attributes = { type: 'rewritten' };
                }
              });
            }

            compilation.hooks.processAssets.tap(
              {
                name: pluginName,
                stage:
                  compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
              },
              () => {
                compilation.emitAsset(
                  'attributes.json',
                  new compiler.rspack.sources.RawSource(
                    JSON.stringify(seen, null, 2),
                  ),
                );
              },
            );
          },
        );
      },
    }),
  ],
});
