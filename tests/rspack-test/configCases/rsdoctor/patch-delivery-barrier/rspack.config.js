const {
  experiments: { RsdoctorPlugin },
} = require('@rspack/core');

const PATCH_HOOKS = [
  'moduleGraph',
  'chunkGraph',
  'moduleIds',
  'moduleSources',
  'assets',
];

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  plugins: [
    new RsdoctorPlugin({
      moduleGraphFeatures: true,
      chunkGraphFeatures: true,
    }),
    {
      apply(compiler) {
        const delivered = new Set();
        let checked = false;

        compiler.hooks.compilation.tap(
          'TestPlugin::PatchDeliveryBarrier',
          (compilation) => {
            const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
            for (const name of PATCH_HOOKS) {
              hooks[name].tapPromise(
                'TestPlugin::PatchDeliveryBarrier',
                async () => {
                  // Patches are sent from a spawned task, so a consumer only observes them
                  // some ticks after the hook that produced them returned. Waiting here makes
                  // the delivery outlive the sealing phase unless it is awaited.
                  await new Promise((resolve) => setTimeout(resolve, 50));
                  delivered.add(name);
                },
              );
            }
          },
        );

        // Rsdoctor finalizes its report on `afterCompile`, so every patch has to be delivered
        // by then.
        compiler.hooks.afterCompile.tap(
          'TestPlugin::PatchDeliveryBarrier',
          () => {
            expect(Array.from(delivered).sort()).toEqual(
              PATCH_HOOKS.slice().sort(),
            );
            checked = true;
          },
        );

        compiler.hooks.done.tap('TestPlugin::PatchDeliveryBarrier', () => {
          expect(checked).toBe(true);
        });
      },
    },
  ],
};
