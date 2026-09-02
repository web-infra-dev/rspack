const pluginName = 'attributes-plugin';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    {
      apply(compiler) {
        /** @type {Record<string, Record<string, unknown>>} */
        const seen = {};

        const record = (hook, resolveData) => {
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
            ]) {
              normalModuleFactory.hooks[hook].tap(pluginName, (resolveData) => {
                record(hook, resolveData);
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
    },
  ],
};
