import assert from 'node:assert/strict';
import type { Compiler, RspackOptions, StatsOptions } from '@rspack/core';

export default {
  context: import.meta.dirname,
  entry: { bundle0: './index.js', other: './other.js' },
  output: { filename: '[name].js' },
  optimization: { concatenateModules: false, minimize: false },
  plugins: [
    {
      apply(compiler: Compiler) {
        compiler.hooks.done.tap('ContextFreeItemsContract', (stats) => {
          assert(
            !stats.hasErrors(),
            stats.toString({ all: false, errors: true }),
          );
          const compilation = stats.compilation;
          const options: StatsOptions = {
            all: false,
            modules: true,
            chunks: true,
            chunkModules: true,
            reasons: true,
            ids: true,
            dependentModules: true,
          };
          const variants = [
            options,
            { ...options, ids: false },
            { ...options, orphanModules: false, reasonsSpace: 1 },
          ];
          const results = variants.map((options) => stats.toJson(options));
          const shared = results[0]
            .chunks!.flatMap((chunk) => chunk.modules!)
            .filter((module) => module.name === './shared.js');
          assert.equal(shared.length, 2, JSON.stringify(results[0].chunks));
          assert(shared.every((module) => module.issuerPath!.length > 0));
          assert.notStrictEqual(shared[0].reasons, shared[1].reasons);
          assert.notStrictEqual(shared[0].reasons![0], shared[1].reasons![0]);
          assert.notStrictEqual(shared[0].issuerPath, shared[1].issuerPath);
          assert.notStrictEqual(
            shared[0].issuerPath![0],
            shared[1].issuerPath![0],
          );

          const factory = compilation.createStatsFactory(
            compilation.createStatsOptions(options, {}),
          );
          const rawChunk = { sizes: [], modules: [] };
          const rawModules = rawChunk.modules;
          const firstChunk = factory.create('chunk', rawChunk, { compilation });
          assert.strictEqual(rawChunk.modules, rawModules);
          assert.equal(
            JSON.stringify(firstChunk),
            JSON.stringify(factory.create('chunk', rawChunk, { compilation })),
          );
          const descriptor = {
            identifier: 'parent',
            name: './parent.js',
            id: 'parent',
          };
          const rawModule = {
            commonAttributes: { reasons: [] },
            issuerDescriptor: descriptor,
            issuerPath: [{ moduleDescriptor: descriptor }],
          };
          const rawIssuerPath = rawModule.issuerPath;
          const firstModule = factory.create('module$visible', rawModule, {
            compilation,
          });
          assert.strictEqual(rawModule.issuerPath, rawIssuerPath);
          assert.equal(
            JSON.stringify(firstModule),
            JSON.stringify(
              factory.create('module$visible', rawModule, { compilation }),
            ),
          );
          factory.hooks.extract
            .for('module$visible')
            .tap('RawSnapshotContract', (_object, module) => {
              assert.strictEqual(module.issuerPath, rawIssuerPath);
            });
          factory.create('module$visible', rawModule, { compilation });

          const publicOptions = compilation.createStatsOptions(
            { all: false, ids: false },
            {},
          );
          const publicFactory = compilation.createStatsFactory(publicOptions);
          publicFactory.hooks.extract
            .for('compilation')
            .tap('PublicRootContext', (object, _compilation, context) => {
              object.extraIssuerPath = publicFactory.create(
                'module.issuerPath',
                [{}],
                context,
              );
            });
          const nativeStats = compilation.__internal_getInner().getStats();
          const rawStats = nativeStats.toJson(publicOptions);
          const lazyFactory = compilation.createStatsFactory(publicOptions);
          lazyFactory.hooks.extract
            .for('compilation')
            .tap('ComputedIssuerContext', (object, _compilation, context) => {
              let registered = false;
              object.computedIssuerPath = lazyFactory.create(
                'module.issuerPath',
                [
                  {
                    get moduleDescriptor() {
                      if (!registered) {
                        registered = true;
                        lazyFactory.hooks.result
                          .for('moduleIssuer')
                          .tap('LateIssuerResult', (issuer) => ({
                            ...issuer,
                            registeredDuringExtract: true,
                          }));
                      }
                      return descriptor;
                    },
                  },
                ],
                context,
              );
            });
          const lazyResult = lazyFactory.create('compilation', compilation, {
            compilation,
            getStatsCompilation: () => rawStats,
            getInner: () => nativeStats,
          });
          assert.equal(
            lazyResult.computedIssuerPath[0].registeredDuringExtract,
            true,
          );
          const publicResult = publicFactory.create(
            'compilation',
            compilation,
            {
              compilation,
              getStatsCompilation: () => rawStats,
              getInner: () => nativeStats,
            },
          );
          assert.deepEqual(publicResult.extraIssuerPath, [{}]);

          compilation.hooks.statsFactory.tap('ForceGenericStats', () => {});
          for (let i = 0; i < variants.length; i++) {
            assert.equal(
              JSON.stringify(results[i]),
              JSON.stringify(stats.toJson(variants[i])),
            );
          }

          let calls = 0;
          compilation.hooks.statsFactory.tap(
            'CustomReasonContext',
            (factory) => {
              factory.hooks.extract
                .for('moduleReason')
                .tap('CustomReasonContext', (object, reason, context) => {
                  assert(context.type.endsWith('.moduleReason'));
                  assert.strictEqual(context[context.type], reason);
                  assert.strictEqual(context.moduleReason, reason);
                  assert(Number.isInteger(context._index));
                  object.customReason = true;
                  calls++;
                });
            },
          );
          const custom = stats.toJson(options);
          assert(calls > 0);
          assert(
            custom.modules!.some((module) =>
              module.reasons?.some((reason) => 'customReason' in reason),
            ),
          );
        });
      },
    },
  ],
} satisfies RspackOptions;
