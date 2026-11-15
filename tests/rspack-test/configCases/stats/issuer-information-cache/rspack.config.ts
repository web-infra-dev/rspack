import assert from 'node:assert/strict';
import path from 'node:path';
import {
  type Compilation,
  type Compiler,
  type LoaderContext,
  NormalModule,
  type RspackOptions,
  type StatsOptions,
} from '@rspack/core';

export default {
  context: import.meta.dirname,
  entry: { bundle0: './index.js' },
  output: { filename: '[name].js' },
  module: { rules: [{ test: /parent\.js$/, use: ['./issuer-loader.cjs'] }] },
  optimization: { concatenateModules: false, minimize: false },
  plugins: [
    {
      apply(compiler: Compiler) {
        const builds = new WeakMap<
          Compilation,
          { imports: number; restoredParent: boolean }
        >();
        compiler.hooks.thisCompilation.tap(
          'IssuerInformationCacheContract',
          (compilation) => {
            const build = { imports: 0, restoredParent: false };
            builds.set(compilation, build);
            NormalModule.getCompilationHooks(compilation).loader.tap(
              'IssuerInformationCacheContract',
              (
                loaderContext: LoaderContext & {
                  onIssuerImport?: () => void;
                },
              ) => {
                loaderContext.onIssuerImport = () => build.imports++;
              },
            );
            compilation.hooks.stillValidModule.tap(
              'IssuerInformationCacheContract',
              (module) => {
                if (
                  module.nameForCondition() ===
                  path.join(compiler.context, 'parent.js')
                ) {
                  build.restoredParent = true;
                }
              },
            );
          },
        );
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
          const build = builds.get(stats.compilation)!;
          assert.equal(
            build.imports,
            build.restoredParent ? 0 : 1,
            'only a restored parent may skip the loader importModule call',
          );
          // Restoring parent skips its loader, so a fresh compiler has no
          // build-time graph. Decide from execution, not missing stats entries.
          assert.equal(
            firstModules.length,
            build.imports === 1 ? 2 : 1,
            'expected a normal graph and a build-time graph only when imported',
          );
          for (const [name, issuerPath] of [
            ['./parent.js', ['./index.js']],
            ['./first.js', ['./index.js', './parent.js']],
            ['./second.js', ['./index.js', './parent.js']],
            ['./third.js', ['./index.js', './parent.js', './first.js']],
          ] as const) {
            const modules = defaultStats[0]
              .chunks!.flatMap((chunk) => chunk.modules!)
              .filter((module) => module.name === name);
            assert.equal(
              modules.length,
              1,
              `expected the normal graph ${name}`,
            );
            assert.equal(modules[0].issuerName, issuerPath.at(-1));
            assert.deepEqual(
              modules[0].issuerPath!.map((issuer) => issuer.name),
              issuerPath,
              `expected the normal issuer path for ${name}`,
            );
          }
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
