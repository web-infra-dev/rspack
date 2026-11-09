import { rspack } from '@rspack/core';

export default {
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: [
          { loader: './loader.js', parallel: { maxWorkers: 1 }, options: {} },
        ],
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        let builtValueModule = false;
        compiler.hooks.compilation.tap('NativeWorkerModule', (compilation) => {
          builtValueModule = false;
          compilation.hooks.processAssets.tap('NativeWorkerModule', () => {
            if (compilation.errors.length > 0) return;
            const asset = compilation.getAsset('worker-module.txt');
            expect(asset.source.source()).toBe('from native worker');
            expect(asset.info.immutable).toBe(true);
            expect(asset.info.sourceFilename).toBe('value.js');
          });
          compilation.hooks.buildModule.tap('NativeWorkerModule', (module) => {
            if (!module.resource?.endsWith('value.js')) return;
            builtValueModule = true;
            module.buildInfo.fromBuildModule = () => 'main buildModule';
          });
          rspack.NormalModule.getCompilationHooks(compilation).loader.tap(
            'NativeWorkerModule',
            (context, module) => {
              if (!module.resource?.endsWith('value.js')) return;
              throw new Error('Parallel loaders must skip the loader hook');
            },
          );
        });
        compiler.hooks.finishMake.tap('NativeWorkerModule', (compilation) => {
          // A restored module skips the loader and uses fresh factory metadata.
          if (!builtValueModule || compilation.errors.length > 0) return;
          const module = [...compilation.modules].find((m) =>
            m.resource?.endsWith('value.js'),
          );
          expect(module.factoryMeta.sideEffectFree).toBe(false);
          expect(module.matchResource).toBe('worker-resource.js');
          expect(module.buildInfo.nativeWorker).toBe(true);
        });
      },
    },
  ],
};
