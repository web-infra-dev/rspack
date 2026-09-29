import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RsdoctorPlugin },
} = rspack;

const PATCH_HOOKS = [
  'moduleGraph',
  'chunkGraph',
  'moduleIds',
  'moduleSources',
  'assets',
] as const;

export default defineConfig({
  mode: 'development',
  plugins: [
    new RsdoctorPlugin({
      moduleGraphFeatures: true,
      chunkGraphFeatures: true,
    }),
    definePlugin({
      apply(compiler) {
        const delivered = new Set<(typeof PATCH_HOOKS)[number]>();
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
    }),
  ],
});
