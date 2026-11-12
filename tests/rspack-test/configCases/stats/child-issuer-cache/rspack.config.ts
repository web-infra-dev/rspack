import assert from 'node:assert/strict';
import path from 'node:path';
import { type Compiler, EntryPlugin, type RspackOptions } from '@rspack/core';

export default {
  context: import.meta.dirname,
  optimization: { concatenateModules: false, moduleIds: 'named' },
  plugins: [
    {
      apply(compiler: Compiler) {
        const childCalls = new Map<string, number>();
        compiler.hooks.thisCompilation.tap(
          'ChildIssuerContract',
          (compilation) => {
            compilation.hooks.statsFactory.tap(
              'ChildIssuerContract',
              (factory) => {
                const seen = new WeakSet<object>();
                const records = new WeakSet<object>();
                factory.hooks.extract
                  .for('module$visible')
                  .tap('ChildIssuerContract', (_object, module, context) => {
                    const issuerPath = module.issuerPath;
                    if (!Array.isArray(issuerPath)) return;
                    assert(
                      !seen.has(issuerPath),
                      'custom hooks must receive independent raw issuer paths',
                    );
                    seen.add(issuerPath);
                    for (const issuer of issuerPath) {
                      assert(
                        !records.has(issuer),
                        'custom hooks must receive independent raw issuer records',
                      );
                      records.add(issuer);
                    }
                    if (
                      !context.compilation.compiler.isChild() ||
                      issuerPath.length === 0
                    ) {
                      return;
                    }
                    const name = module.commonAttributes.moduleDescriptor?.name;
                    assert(name === './first.js' || name === './second.js');
                    assert.deepEqual(
                      issuerPath.map(({ moduleDescriptor }) => ({
                        identifier: path.relative(
                          compiler.context,
                          moduleDescriptor.identifier,
                        ),
                        name: moduleDescriptor.name,
                        id: moduleDescriptor.id,
                      })),
                      [
                        {
                          identifier: 'child-entry.js',
                          name: './child-entry.js',
                          id: './child-entry.js',
                        },
                      ],
                    );
                    childCalls.set(name, (childCalls.get(name) ?? 0) + 1);
                  });
              },
            );
          },
        );
        compiler.hooks.make.tapAsync(
          'ChildIssuerContract',
          (compilation, callback) => {
            const child = compilation.createChildCompiler(
              'stats-child',
              { filename: 'child.js' },
              [
                new EntryPlugin(compiler.context, './child-entry.js', {
                  name: 'child',
                }),
              ],
            );
            child.runAsChild((error) => callback(error));
          },
        );
        compiler.hooks.done.tap('ChildIssuerContract', (stats) => {
          assert(
            !stats.hasErrors(),
            stats.toString({ all: false, errors: true }),
          );
          stats.toJson({
            all: false,
            modules: true,
            chunks: true,
            chunkModules: true,
            children: true,
            cachedModules: true,
            dependentModules: true,
            ids: true,
          });
          for (const name of ['./first.js', './second.js']) {
            assert(
              (childCalls.get(name) ?? 0) >= 2,
              `expected non-empty child issuer paths in module and chunk stats for ${name}`,
            );
          }
        });
      },
    },
  ],
} satisfies RspackOptions;
