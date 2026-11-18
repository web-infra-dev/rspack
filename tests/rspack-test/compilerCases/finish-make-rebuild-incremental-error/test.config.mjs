import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { NormalModule } from '@rspack/core';

const cases = [];
for (const stage of ['make', 'rebuild']) {
  for (const cache of [false, true]) {
    const state = { fail: false };
    const built = [];
    const explicit = stage === 'rebuild';
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
        }))
          fs.writeFileSync(path.join(input, name), source);
        return {
          context: input,
          mode: 'development',
          devtool: false,
          cache,
          incremental: { buildChunkGraph: true },
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
          });
        });
        compiler.hooks.finishMake.tapPromise('Test', async (compilation) => {
          if (!state.fail) return;
          if (!explicit) return;
          const module = [...compilation.modules].find(
            (module) => path.basename(module.resource || '') === 'changed.js',
          );
          const error = await new Promise((resolve) =>
            compilation.rebuildModule(module, resolve),
          );
          expect(error).toBeNull();
        });
      },
      async build(context, compiler) {
        const initial = await new Promise((resolve) => compiler.run(resolve));
        expect(initial).toBeNull();
        const changed = context.getDist(path.join('input', 'changed.js'));
        fs.writeFileSync(changed, "export default 'updated';");
        const rebuild = (files) =>
          new Promise((resolve) =>
            compiler.__internal__rebuild(new Set(files), new Set(), resolve),
          );
        built.length = 0;
        state.fail = true;
        const failed = await rebuild(explicit ? [] : [changed]);
        expect(failed).toBeFalsy();
        expect(compiler._lastCompilation.errors.length).toBeGreaterThan(0);
        expect(built).not.toContain('stable.js');
        expect(built).not.toContain('payload.js');

        const repeated = await rebuild([]);
        expect(repeated).toBeFalsy();
        expect(compiler._lastCompilation.errors.length).toBeGreaterThan(0);
        expect(built).not.toContain('stable.js');
        expect(built).not.toContain('payload.js');

        state.fail = false;
        built.length = 0;
        // Both normal make and rebuildModule retry modules with build errors
        // without rebuilding their unaffected neighbors.
        expect(await rebuild([])).toBeFalsy();
        expect(built).not.toContain('stable.js');
        expect(built).not.toContain('payload.js');
        expect(built).not.toContain('entry.js');
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
