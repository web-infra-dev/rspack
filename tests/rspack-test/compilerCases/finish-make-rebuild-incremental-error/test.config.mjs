import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { NormalModule } from '@rspack/core';

const cases = [];
for (const stage of [
  'make',
  'buildModule',
  'succeedModule',
  'factorize',
  'new-dependency',
  'new-factorize',
  'executor',
  'finishMake',
  'rebuildModule',
  'finishModules',
  'optimizeModules',
  'processAssets',
  'loader-make',
  'loader-rebuild',
]) {
  for (const cache of [false, true]) {
    const state = {
      fail: false,
      rebuilding: false,
      loaderError: false,
      executorError: stage === 'executor',
    };
    const built = [];
    const explicit = stage === 'rebuildModule' || stage === 'loader-rebuild';
    const diagnostic = stage.startsWith('loader-');
    const hookError = () => {
      throw new Error('expected interrupted update');
    };
    cases.push({
      description: `finishMake preserves unrelated modules after ${stage} errors, cache=${cache}`,
      options(context) {
        const input = context.getDist('input');
        fs.mkdirSync(input, { recursive: true });
        for (const [name, source] of Object.entries({
          'entry.js':
            "import changed from './changed.js'; import stable from './stable.js'; export default [changed, stable];",
          'changed.js': "export default 'initial';",
          'stable.js': '',
          'payload.js': "export default 'stable';",
          'added.js': "export default 'updated';",
          'executed.js': "export default 'unused';",
        }))
          fs.writeFileSync(path.join(input, name), source);
        return {
          context: input,
          mode: 'development',
          devtool: false,
          cache,
          bail: stage === 'factorize' || stage === 'new-factorize',
          incremental: { buildChunkGraph: cache },
          entry: './entry.js',
          output: {
            path: context.getDist('output'),
            filename: 'main.js',
            library: { type: 'commonjs2' },
          },
          module: {
            rules: [
              { test: /changed\.js$/, use: context.getSource('loader.cjs') },
              {
                test: /stable\.js$/,
                use: context.getSource('import-loader.cjs'),
              },
            ],
          },
        };
      },
      async compiler(context, compiler) {
        compiler.hooks.normalModuleFactory.tap('Test', (factory) => {
          factory.hooks.beforeResolve.tap('Test', (data) => {
            if (
              state.fail &&
              ((stage === 'factorize' && data.request === './changed.js') ||
                (stage === 'new-factorize' && data.request === './added.js'))
            )
              hookError();
          });
        });
        compiler.hooks.thisCompilation.tap('Test', (compilation) => {
          NormalModule.getCompilationHooks(compilation).loader.tap(
            'Test',
            (loader) => {
              loader.recoveryState = state;
            },
          );
          compilation.hooks.buildModule.tap('Test', (module) => {
            const name = path.basename(module.resource || '');
            built.push(name);
            if (
              state.fail &&
              name === 'changed.js' &&
              (stage === 'buildModule' ||
                (stage === 'rebuildModule' && state.rebuilding))
            )
              hookError();
          });
          compilation.hooks.succeedModule.tap('Test', (module) => {
            if (
              state.fail &&
              ((stage === 'succeedModule' &&
                path.basename(module.resource || '') === 'changed.js') ||
                (stage === 'new-dependency' &&
                  path.basename(module.resource || '') === 'added.js') ||
                (stage === 'executor' &&
                  path.basename(module.resource || '') === 'executed.js'))
            )
              hookError();
          });
          for (const hook of [
            'finishModules',
            'optimizeModules',
            'processAssets',
          ]) {
            compilation.hooks[hook].tap('Test', () => {
              if (state.fail && stage === hook) hookError();
            });
          }
        });
        compiler.hooks.make.tap('Test', () => {
          if (state.fail && stage === 'make') hookError();
        });
        compiler.hooks.finishMake.tapPromise('Test', async (compilation) => {
          if (!state.fail) return;
          if (stage === 'finishMake') hookError();
          if (!explicit) return;
          const module = [...compilation.modules].find(
            (module) => path.basename(module.resource || '') === 'changed.js',
          );
          state.rebuilding = true;
          const error = await new Promise((resolve) =>
            compilation.rebuildModule(module, resolve),
          );
          state.rebuilding = false;
          if (!diagnostic)
            expect(error.message).toContain('expected interrupted update');
        });
      },
      async build(context, compiler) {
        const initial = await new Promise((resolve) => compiler.run(resolve));
        expect(initial).toBeNull();
        const changed = context.getDist(path.join('input', 'changed.js'));
        fs.writeFileSync(
          changed,
          stage.startsWith('new-')
            ? "export { default } from './added.js';"
            : "export default 'updated';",
        );
        const rebuild = (files) =>
          new Promise((resolve) =>
            compiler.__internal__rebuild(new Set(files), new Set(), resolve),
          );
        built.length = 0;
        state.fail = true;
        state.loaderError = diagnostic;
        const failed = await rebuild(explicit ? [] : [changed]);
        if (diagnostic) {
          expect(failed).toBeFalsy();
          expect(compiler._lastCompilation.errors.length).toBeGreaterThan(0);
        } else {
          expect(failed.message).toContain('expected interrupted update');
        }
        expect(built).not.toContain('stable.js');
        expect(built).not.toContain('payload.js');

        const repeated = await rebuild([]);
        if (diagnostic) {
          expect(repeated).toBeFalsy();
          expect(compiler._lastCompilation.errors.length).toBeGreaterThan(0);
        } else {
          expect(repeated.message).toContain('expected interrupted update');
        }
        expect(built).not.toContain('stable.js');
        expect(built).not.toContain('payload.js');

        state.fail = false;
        state.loaderError = false;
        built.length = 0;
        // No repeated file-change notification: interrupted work must survive.
        expect(await rebuild([])).toBeFalsy();
        expect(built).not.toContain('stable.js');
        expect(built).not.toContain('payload.js');
        expect(built).not.toContain('entry.js');
        if (stage.startsWith('new-')) {
          // Its new child failed, but the parent module finished building.
          expect(built).not.toContain('changed.js');
        }
        if (
          [
            'finishMake',
            'finishModules',
            'optimizeModules',
            'processAssets',
          ].includes(stage)
        ) {
          expect(built).toEqual([]);
        }
        const compilation = compiler._lastCompilation;
        expect(compilation.errors).toHaveLength(0);
        expect(
          compilation.getAsset('stable.txt').source.source().toString(),
        ).toBe('stable');
        expect(
          compilation.fileDependencies.has(
            context.getDist(path.join('input', 'payload.js')),
          ),
        ).toBe(true);
        const module = { exports: {} };
        vm.runInNewContext(
          compilation.getAsset('main.js').source.source().toString(),
          { module },
        );
        expect(module.exports.default).toEqual(['updated', 'stable']);

        built.length = 0;
        expect(await rebuild([])).toBeFalsy();
        expect(built).toEqual([]);
      },
    });
  }
}
export default cases;
