import assert from 'node:assert/strict';
import { type Compiler, EntryPlugin, type RspackOptions } from '@rspack/core';

export default {
  context: import.meta.dirname,
  plugins: [
    {
      apply(compiler: Compiler) {
        let childCalls = 0;
        compiler.hooks.thisCompilation.tap(
          'ChildIssuerContract',
          (compilation) => {
            compilation.hooks.statsFactory.tap(
              'ChildIssuerContract',
              (factory) => {
                const seen = new WeakSet<object>();
                factory.hooks.extract
                  .for('module$visible')
                  .tap('ChildIssuerContract', (_object, module, context) => {
                    if (!Array.isArray(module.issuerPath)) return;
                    assert(
                      !seen.has(module.issuerPath),
                      'custom hooks must receive independent raw issuer paths',
                    );
                    seen.add(module.issuerPath);
                    if (context.compilation.compiler.isChild()) childCalls++;
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
                new EntryPlugin(compiler.context, './index.js', {
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
          assert(childCalls >= 2);
        });
      },
    },
  ],
} satisfies RspackOptions;
