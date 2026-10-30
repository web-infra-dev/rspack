import assert from 'node:assert/strict';
import type { Compiler, RspackOptions, StatsOptions } from '@rspack/core';

export default {
  context: import.meta.dirname,
  entry: { bundle0: './index.js' },
  output: { filename: '[name].js' },
  module: { rules: [{ test: /parent\.js$/, use: ['./issuer-loader.cjs'] }] },
  optimization: { concatenateModules: false, minimize: false },
  plugins: [
    {
      apply(compiler: Compiler) {
        compiler.hooks.done.tap('IssuerInformationCacheContract', (stats) => {
          assert(
            !stats.hasErrors(),
            stats.toString({ all: false, errors: true }),
          );
          const options: StatsOptions = {
            all: false,
            modules: true,
            chunks: true,
            chunkModules: true,
            dependentModules: true,
            orphanModules: true,
            cachedModules: true,
            reasons: true,
            ids: true,
          };
          const variants = [options, { ...options, ids: false }];
          const defaultStats = variants.map((options) => stats.toJson(options));
          const expected = defaultStats.map((result) => JSON.stringify(result));
          const firstModules = defaultStats[0].modules!.filter(
            (module) => module.name === './first.js',
          );
          assert(
            firstModules.length >= 2,
            'expected the same module identifier in normal and build-time graphs',
          );
          let sharedIssuerRecords = 0;
          stats.compilation.hooks.statsFactory.tap(
            'IssuerInformationCacheContract',
            (factory) => {
              const paths = new WeakSet<object>();
              const records = new WeakSet<object>();
              const names = new Map<string, string>();
              factory.hooks.extract
                .for('module$visible')
                .tap('IssuerInformationCacheContract', (_object, module) => {
                  if (!module.issuerPath) return;
                  assert(
                    !paths.has(module.issuerPath),
                    'raw issuer path arrays must stay independent',
                  );
                  paths.add(module.issuerPath);
                  for (const issuer of module.issuerPath) {
                    assert(
                      !records.has(issuer),
                      'raw issuer records must stay independent',
                    );
                    records.add(issuer);
                    const descriptor = issuer.moduleDescriptor;
                    if (names.has(descriptor.identifier)) {
                      assert.equal(
                        descriptor.name,
                        names.get(descriptor.identifier),
                      );
                      sharedIssuerRecords++;
                    } else {
                      names.set(descriptor.identifier, descriptor.name);
                    }
                  }
                });
            },
          );
          for (let index = 0; index < variants.length; index++) {
            assert.equal(
              JSON.stringify(stats.toJson(variants[index])),
              expected[index],
            );
          }
          assert(
            sharedIssuerRecords >= 4,
            'expected the same issuer in multiple module paths',
          );
        });
      },
    },
  ],
} satisfies RspackOptions;
