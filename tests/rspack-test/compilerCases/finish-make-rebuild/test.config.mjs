import path from 'node:path';
import vm from 'node:vm';
import { createRequire } from 'node:module';
import { CssExtractRspackPlugin, NormalModule } from '@rspack/core';

const require = createRequire(import.meta.url);

const run = (compiler) =>
  new Promise((resolve) => {
    compiler.run((error, stats) =>
      resolve({ error, compilation: stats?.compilation }),
    );
  });
const rebuild = (compiler) =>
  new Promise((resolve) => {
    compiler.__internal__rebuild(new Set(), new Set(), (error) => {
      resolve({ error, compilation: compiler._lastCompilation });
    });
  });
const request = (compilation, module) =>
  new Promise((resolve) => {
    compilation.rebuildModule(module, (error, result) =>
      resolve({ error, module: result }),
    );
  });
function value(compilation) {
  expect(compilation.errors).toHaveLength(0);
  const module = { exports: {} };
  vm.runInNewContext(
    compilation.getAsset('main.js').source.source().toString(),
    { module },
  );
  return module.exports.default;
}
function options(context, cache = true, incremental = true) {
  return {
    context: context.getSource(),
    mode: 'development',
    devtool: false,
    cache,
    incremental,
    entry: './entry.js',
    output: {
      path: context.getDist(),
      filename: 'main.js',
      library: { type: 'commonjs2' },
    },
  };
}

const cases = [];
for (const kind of ['rule-gained', 'rule-lost', 'removed', 'added']) {
  for (const incremental of [true, false]) {
    const state = {
      round: 0,
      rule: kind === 'rule-lost',
      drop: false,
      extra: false,
    };
    cases.push({
      description: `finishMake completes ordered and duplicate requests: ${kind}, incremental=${incremental}`,
      options(context) {
        const loader = context.getSource('loader.cjs');
        return {
          ...options(context, false, incremental),
          module: {
            rules: [
              { test: /\.js$/, use: loader },
              {
                test: (file) => state.rule && path.basename(file) === 'a.js',
                use: loader,
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
              loader.rebuildState = state;
            },
          );
        });
        compiler.hooks.finishMake.tapPromise('Test', async (compilation) => {
          if (state.round !== 1) return;
          state.rule = kind === 'rule-gained';
          state.drop = kind === 'removed';
          state.extra = kind === 'added';
          const modules = new Map(
            [...compilation.modules].map((m) => [
              path.basename(m.resource || ''),
              m,
            ]),
          );
          const names =
            kind === 'removed'
              ? ['k.js', 'entry.js', 'h.js', 'a.js', 'k.js']
              : ['k.js', 'a.js', 'k.js'];
          const results = await Promise.all(
            names.map((name) => request(compilation, modules.get(name))),
          );
          results.forEach((result, index) => {
            if (kind === 'removed' && names[index] === 'h.js') {
              expect(result.error.message).toContain(
                'no longer in the module graph',
              );
              expect(result.module).toBeNull();
            } else {
              expect(result.error).toBeNull();
              expect(path.basename(result.module.resource)).toBe(names[index]);
            }
          });
        });
      },
      async build(context, compiler) {
        const initial = await run(compiler);
        expect(initial.error).toBeNull();
        expect(value(initial.compilation)).toEqual(['h', 'k']);
        state.round = 1;
        const updated = await rebuild(compiler);
        expect(updated.error).toBeFalsy();
        const expected =
          kind === 'removed' ? 'removed' : kind === 'added' ? 'extra' : 'h';
        expect(value(updated.compilation)).toEqual([expected, 'k']);
        if (kind === 'added') {
          expect(
            updated.compilation.fileDependencies.has(
              context.getSource('extra.js'),
            ),
          ).toBe(true);
        }
        if (kind === 'removed') {
          expect(
            updated.compilation.fileDependencies.has(context.getSource('h.js')),
          ).toBe(false);
        }
        state.round = 2;
        const next = await rebuild(compiler);
        expect(next.error).toBeFalsy();
        expect(value(next.compilation)).toEqual([expected, 'k']);
      },
    });
  }
}

for (const hook of ['make', 'finishMake', 'rebuildModule']) {
  for (const failFirst of [true, false]) {
    for (const [cache, buildModuleGraph] of [
      [true, true],
      [false, true],
      [true, false],
      [false, false],
    ]) {
      let fail = false;
      let throwBuild = false;
      cases.push({
        description: `finishMake recovers after ${hook} fails on ${failFirst ? 'cold' : 'hot'} build, cache=${cache}, buildModuleGraph=${buildModuleGraph}`,
        options: (context) => options(context, cache, { buildModuleGraph }),
        async compiler(context, compiler) {
          compiler.hooks.thisCompilation.tap('Test', (compilation) => {
            compilation.hooks.buildModule.tap('Test', () => {
              if (throwBuild) {
                throwBuild = false;
                throw new Error('rebuild failed');
              }
            });
          });
          compiler.hooks.make.tap('Test', () => {
            if (fail && hook === 'make') throw new Error('make failed');
          });
          compiler.hooks.finishMake.tapPromise('Test', async (compilation) => {
            if (!fail) return;
            if (hook === 'finishMake') throw new Error('finishMake failed');
            if (hook !== 'rebuildModule') return;
            const module = [...compilation.modules].find(
              (m) => path.basename(m.resource || '') === 'a.js',
            );
            throwBuild = true;
            const result = await request(compilation, module);
            expect(result.error.message).toContain('rebuild failed');
            // Deliberately swallow the callback error. The incomplete graph must
            // still fail this compilation, and the next build must retry it.
          });
        },
        async build(context, compiler) {
          if (!failFirst) {
            expect((await run(compiler)).error).toBeNull();
          }
          fail = true;
          const failed = await (failFirst ? run(compiler) : rebuild(compiler));
          expect(failed.error.message).toContain(
            hook === 'rebuildModule' ? 'rebuild failed' : `${hook} failed`,
          );
          fail = false;
          for (let i = 0; i < 2; i++) {
            const recovered = await rebuild(compiler);
            expect(recovered.error).toBeFalsy();
            expect(value(recovered.compilation)).toEqual(['h', 'k']);
          }
        },
      });
    }
  }
}

for (const cache of [true, false]) {
  cases.push({
    description: `finishMake preserves codegen reuse with buildModuleGraph disabled, cache=${cache}`,
    options: (context) => options(context, cache, { buildModuleGraph: false }),
    async build(context, compiler) {
      const initial = await run(compiler);
      expect(initial.error).toBeNull();
      expect(value(initial.compilation)).toEqual(['h', 'k']);
      const modules = (compilation) =>
        compilation
          .getStats()
          .toJson({
            all: false,
            modules: true,
            cachedModules: true,
          })
          .modules.filter((module) => module.name?.endsWith('.js'));
      const initialModules = modules(initial.compilation);
      expect(initialModules.length).toBeGreaterThan(0);
      expect(initialModules.every((module) => module.codeGenerated)).toBe(true);

      for (let i = 0; i < 2; i++) {
        const rebuilt = await rebuild(compiler);
        expect(rebuilt.error).toBeFalsy();
        expect(value(rebuilt.compilation)).toEqual(['h', 'k']);
        const rebuiltModules = modules(rebuilt.compilation);
        expect(rebuiltModules).toHaveLength(initialModules.length);
        expect(
          rebuiltModules.filter((module) => module.codeGenerated),
        ).toHaveLength(0);
      }
    },
  });
}

cases.push({
  description:
    'finishMake returns a callback error for a rebuild requested in finishModules',
  options,
  async compiler(context, compiler) {
    compiler.hooks.thisCompilation.tap('Test', (compilation) => {
      compilation.hooks.finishModules.tapPromise('Test', async (modules) => {
        const module = [...modules].find(
          (m) => path.basename(m.resource || '') === 'a.js',
        );
        const result = await request(compilation, module);
        expect(result.error.message).toContain('compiler.hooks.finishMake');
      });
    });
  },
  async build(context, compiler) {
    const initial = await run(compiler);
    expect(initial.error).toBeNull();
    expect(value(initial.compilation)).toEqual(['h', 'k']);
    const next = await rebuild(compiler);
    expect(next.error).toBeFalsy();
    expect(value(next.compilation)).toEqual(['h', 'k']);
  },
});

for (const cache of [true, false]) {
  for (const versionedRequest of [true, false]) {
    const state = { phase: 0, drop: false, versionedRequest };
    cases.push({
      description: `finishMake replaces rebuilt executor contributions and keeps other imports, cache=${cache}, versionedRequest=${versionedRequest}`,
      options(context) {
        return {
          ...options(context, cache),
          entry: './import-entry.js',
          module: {
            rules: [
              {
                test: /(?:imports|other)\.js$/,
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
              loader.rebuildState = state;
            },
          );
        });
        compiler.hooks.finishMake.tapPromise('Test', async (compilation) => {
          for (let i = 0; i < 2; i++) {
            state.phase++;
            const module = [...compilation.modules].find(
              (m) => path.basename(m.resource || '') === 'imports.js',
            );
            const result = await request(compilation, module);
            expect(result.error).toBeNull();
          }
        });
      },
      async build(context, compiler) {
        for (let round = 0; round < 3; round++) {
          const result = await (round === 0
            ? run(compiler)
            : rebuild(compiler));
          expect(result.error).toBeFalsy();
          const compilation = result.compilation;
          expect(value(compilation)).toEqual([state.phase, 100]);
          const assets = compilation
            .getAssets()
            .map((asset) => asset.name)
            .sort();
          expect(assets).toEqual(
            [`main-${state.phase}.txt`, 'main.js', 'other-100.txt'].sort(),
          );
          const warnings = compilation.warnings
            .map((warning) => warning.message)
            .join('\n');
          expect(compilation.warnings).toHaveLength(2);
          expect(warnings).toContain(`main warning ${state.phase}`);
          expect(warnings).toContain('other warning 100');
          expect(
            compilation.fileDependencies.has(context.getSource('payload.js')),
          ).toBe(true);
        }
        state.drop = true;
        for (let round = 0; round < 2; round++) {
          const result = await rebuild(compiler);
          expect(result.error).toBeFalsy();
          expect(value(result.compilation)).toEqual([-1, 100]);
          expect(
            result.compilation
              .getAssets()
              .map((asset) => asset.name)
              .sort(),
          ).toEqual(['main.js', 'other-100.txt']);
          expect(result.compilation.warnings).toHaveLength(1);
          expect(
            result.compilation.fileDependencies.has(
              context.getSource('payload.js'),
            ),
          ).toBe(false);
          expect(
            result.compilation.fileDependencies.has(
              context.getSource('other-payload.js'),
            ),
          ).toBe(true);
        }
      },
    });
  }
}

const executorState = { phase: 0, fail: true };
cases.push({
  description:
    'finishMake releases pending importModule callbacks when the executor fails',
  options(context) {
    return {
      ...options(context),
      entry: './import-entry.js',
      module: {
        rules: [
          {
            test: /(?:imports|other)\.js$/,
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
          loader.rebuildState = executorState;
        },
      );
      compilation.hooks.buildModule.tap('Test', (module) => {
        if (
          executorState.fail &&
          path.basename(module.resource || '') === 'payload.js'
        ) {
          throw new Error('executor build failed');
        }
      });
    });
  },
  async build(context, compiler) {
    const failed = await run(compiler);
    expect(failed.error.message).toContain('executor build failed');
    executorState.fail = false;
    const recovered = await rebuild(compiler);
    expect(recovered.error).toBeFalsy();
    expect(value(recovered.compilation)).toEqual([0, 100]);
  },
});

cases.push({
  description:
    'finishMake can add nested importModule requests while rebuilding',
  options(context) {
    return {
      ...options(context),
      entry: './payload.js',
      module: {
        rules: [
          {
            test: /payload\.js$/,
            use: `${context.getSource('nested-loader.cjs')}?depth=2`,
          },
        ],
      },
    };
  },
  async compiler(context, compiler) {
    compiler.hooks.finishMake.tapPromise('Test', async (compilation) => {
      const module = [...compilation.modules].find(
        (m) => path.basename(m.resource || '') === 'payload.js',
      );
      expect((await request(compilation, module)).error).toBeNull();
    });
  },
  async build(context, compiler) {
    for (let round = 0; round < 3; round++) {
      const result = await (round === 0 ? run(compiler) : rebuild(compiler));
      expect(result.error).toBeFalsy();
      expect(value(result.compilation)).toBe(44);
    }
  },
});

for (const incremental of [true, false]) {
  for (const cache of [true, false]) {
    for (const shared of [true, false]) {
      const state = { phase: 0 };
      cases.push({
        description: `finishMake refreshes extracted CSS across repeated rebuilds, incremental=${incremental}, cache=${cache}, shared=${shared}`,
        options(context) {
          const loaders = [
            CssExtractRspackPlugin.loader,
            require.resolve('css-loader'),
            context.getSource('style-loader.cjs'),
          ];
          return {
            ...options(context, cache, incremental),
            entry: shared
              ? [
                  './style-entry.js',
                  `!!${context.getSource('passthrough-loader.cjs')}!${loaders.join('!')}!${context.getSource('style.css')}`,
                ]
              : './style-entry.js',
            module: {
              rules: [
                {
                  test: /\.css$/,
                  type: 'javascript/auto',
                  use: loaders,
                },
              ],
            },
            plugins: [new CssExtractRspackPlugin({ filename: 'main.css' })],
          };
        },
        async compiler(context, compiler) {
          compiler.hooks.thisCompilation.tap('Test', (compilation) => {
            NormalModule.getCompilationHooks(compilation).loader.tap(
              'Test',
              (loader) => {
                loader.rebuildState = state;
              },
            );
          });
          compiler.hooks.finishMake.tapPromise('Test', async (compilation) => {
            for (let i = 0; i < 2; i++) {
              state.phase++;
              const module = [...compilation.modules].find(
                (m) =>
                  m.type === 'javascript/auto' &&
                  path.basename(m.resource || '') === 'style.css',
              );
              expect((await request(compilation, module)).error).toBeNull();
            }
          });
        },
        async build(context, compiler) {
          let identifier;
          for (let round = 0; round < 3; round++) {
            const result = await (round === 0
              ? run(compiler)
              : rebuild(compiler));
            expect(result.error).toBeFalsy();
            expect(result.compilation.errors).toHaveLength(0);
            const css = result.compilation
              .getAsset('main.css')
              .source.source()
              .toString();
            expect(css).toContain(`color: rgb(${state.phase}, 0, 0)`);
            expect(css).not.toContain(`color: rgb(${state.phase - 1}, 0, 0)`);
            const modules = [...result.compilation.modules].filter(
              (m) => m.type === 'css/mini-extract',
            );
            expect(modules).toHaveLength(1);
            identifier ??= modules[0].identifier();
            expect(modules[0].identifier()).toBe(identifier);
          }
        },
      });
    }
  }
}

export default cases;
