import path from 'node:path';
import vm from 'node:vm';
import { NormalModule } from '@rspack/core';

function deferred() {
  let resolve;
  const promise = new Promise((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

export default ['make', 'rebuild', 'executor-late', 'executor-pending'].map(
  (stage) => {
    const executor = stage.startsWith('executor');
    const state = {
      failing: stage !== 'rebuild',
      rebuildPending: stage === 'rebuild',
      nestedBeforeFailure: stage === 'executor-pending',
      bStarted: deferred(),
      nestedStarted: deferred(),
      failureObserved: deferred(),
      bFinished: false,
      payloadFinished: false,
      imported: undefined,
      importError: undefined,
    };
    return {
      description: `finishMake drains active loaders after ${stage} fails`,
      options(context) {
        return {
          context: context.getSource(),
          mode: 'development',
          cache: true,
          incremental: true,
          devtool: false,
          entry: executor ? './outer.js' : './entry.js',
          output: {
            path: context.getDist(),
            filename: 'main.js',
            library: { type: 'commonjs2' },
          },
          module: {
            rules: [
              {
                test: /(?:a|b|outer)\.js$/,
                use: context.getSource('loader.cjs'),
              },
            ],
          },
        };
      },
      async compiler(context, compiler) {
        compiler.hooks.thisCompilation.tap('Test', (compilation) => {
          NormalModule.getCompilationHooks(compilation).loader.tap(
            'Test',
            (loader) => {
              loader.cleanupState = state;
            },
          );
          compilation.hooks.succeedModule.tap('Test', (module) => {
            if (
              state.failing &&
              path.basename(module.resource || '') === 'a.js'
            ) {
              state.failureObserved.resolve();
              throw new Error('pending loader failure');
            }
          });
        });
        compiler.hooks.finishMake.tapPromise('Test', async (compilation) => {
          if (!state.rebuildPending) return;
          state.rebuildPending = false;
          state.failing = true;
          const modules = [...compilation.modules].filter((module) =>
            ['a.js', 'b.js'].includes(path.basename(module.resource || '')),
          );
          await Promise.all(
            modules.map(
              (module) =>
                new Promise((resolve) => {
                  compilation.rebuildModule(module, (error) => {
                    expect(error.message).toContain('pending loader failure');
                    resolve();
                  });
                }),
            ),
          );
        });
      },
      async build(context, compiler) {
        const error = await new Promise((resolve) => compiler.run(resolve));
        expect(error.message).toContain('pending loader failure');
        // The error callback is also the boundary for starting another build.
        expect(state.bFinished).toBe(true);
        if (executor) {
          expect(state.importError.message).toContain(
            'Module executor stopped',
          );
        } else {
          expect(state.imported).toBe(42);
        }
        if (!executor || state.nestedBeforeFailure) {
          expect(state.payloadFinished).toBe(true);
        }

        state.failing = false;
        const recovered = await new Promise((resolve) => {
          compiler.__internal__rebuild(new Set(), new Set(), resolve);
        });
        expect(recovered).toBeFalsy();
        const compilation = compiler._lastCompilation;
        expect(compilation.errors).toHaveLength(0);
        expect(compilation.getAsset('failed-import.txt')).toBeUndefined();
        expect(
          compilation.fileDependencies.has(context.getSource('payload.js')),
        ).toBe(false);
        const module = { exports: {} };
        vm.runInNewContext(
          compilation.getAsset('main.js').source.source().toString(),
          {
            module,
          },
        );
        expect(module.exports.default).toEqual(['a', 'b']);
      },
    };
  },
);
