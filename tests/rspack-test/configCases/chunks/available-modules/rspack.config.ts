import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';
import { experiments } from '@rspack/core';

const scenarios = [
  'late-addition',
  'late-and-new-child',
  'early-addition',
  'preserve-mask',
  'grow-then-shrink',
  'multiple-parents',
  'cycle',
  'multiple-runtimes',
  'worker-boundary',
  'shared-physical-chunk',
  'two-growing-parents',
  'inherited-growth',
  'late-addition-with-restore',
  'late-shared-physical-chunk',
  'late-worker-addition',
];
export default scenarios.map((scenario, index) => {
  const retains = [
    'grow-then-shrink',
    'multiple-parents',
    'multiple-runtimes',
    'worker-boundary',
  ].includes(scenario);
  const modules = {
    'index.js': `
      ${scenario === 'late-addition-with-restore' ? "export const direct = async () => (await import(/* webpackChunkName: 'root' */ './root')).load();" : "export const direct = () => import(/* webpackChunkName: 'parent' */ './parent-a');"}
      export const indirect = () => import(/* webpackChunkName: 'q' */ './q');
      ${scenario === 'early-addition' ? "export const early = () => import(/* webpackChunkName: 'parent' */ './parent-b');" : ''}
      ${scenario === 'multiple-parents' ? "export const child = () => import(/* webpackChunkName: 'child' */ './child');" : ''}
      ${scenario === 'worker-boundary' || scenario === 'shared-physical-chunk' ? `export const worker = () => new Worker(/* webpackChunkName: '${scenario === 'shared-physical-chunk' ? 'parent' : 'worker'}' */ new URL('./worker', import.meta.url));` : ''}
      ${scenario === 'late-worker-addition' ? "export const worker = () => new Worker(/* webpackChunkName: 'worker' */ new URL('./worker-a', import.meta.url));" : ''}
      ${scenario === 'two-growing-parents' ? "export const second = () => import(/* webpackChunkName: 'parent2' */ './parent2-a');" : ''}
      it('maintains available modules: ${scenario}', async () => {
        // First load the direct path, before executing the late named import.
        ${scenario === 'preserve-mask' ? 'expect((await direct()).value).toBe(42);' : scenario === 'inherited-growth' ? 'expect((await (await (await direct()).load()).load()).value).toBe(42);' : 'expect((await (await direct()).load()).value).toBe(42);'}
        ${scenario === 'two-growing-parents' ? 'expect((await (await second()).load()).value).toBe(42);' : ''}
        const r = await (await indirect()).load();
        const addition = await r.load();
        ${scenario === 'preserve-mask' ? 'expect((await addition.load()).value).toBe(42);' : 'expect(addition.value).toBe(42);'}
        ${scenario === 'grow-then-shrink' ? 'expect((await (await (await r.later()).load()).load()).value).toBe(42);' : ''}
        ${scenario === 'two-growing-parents' ? 'expect((await (await r.later()).load()).value).toBe(42);' : ''}
      });
    `,
    'parent-a.js':
      scenario === 'preserve-mask'
        ? "export { value } from './m';"
        : `${scenario === 'late-addition-with-restore' ? "export { value } from './x';" : ''}
          export const load = () => import(/* webpackChunkName: 'child' */ './child');`,
    'root.js': `export { value } from './x';
      export const load = () => import(/* webpackChunkName: 'parent' */ './parent-a');`,
    'x.js': 'export const value = 43;',
    'q.js':
      "export const load = () => import(/* webpackChunkName: 'r' */ './r');",
    'r.js': `export const load = () => import(/* webpackChunkName: 'parent' */ './parent-b');
      ${scenario === 'late-shared-physical-chunk' ? "export const worker = () => new Worker(/* webpackChunkName: 'parent' */ new URL('./worker', import.meta.url));" : ''}
      ${scenario === 'late-worker-addition' ? "export const worker = () => new Worker(/* webpackChunkName: 'worker' */ new URL('./worker-b', import.meta.url));" : ''}
      ${scenario === 'grow-then-shrink' || scenario === 'two-growing-parents' ? "export const later = () => import(/* webpackChunkName: 's' */ './s');" : ''}`,
    's.js':
      scenario === 'two-growing-parents'
        ? "export const load = () => import(/* webpackChunkName: 'parent2' */ './parent2-b');"
        : "export const load = () => import(/* webpackChunkName: 't' */ './t');",
    'parent2-a.js':
      "export const load = () => import(/* webpackChunkName: 'child' */ './child');",
    'parent2-b.js': "export { value } from './m';",
    'grandchild.js': "export { value } from './m';",
    't.js':
      "export const load = () => import(/* webpackChunkName: 'child' */ './child');",
    'parent-b.js':
      scenario === 'preserve-mask'
        ? "export const load = () => import(/* webpackChunkName: 'child' */ './child');"
        : scenario === 'late-shared-physical-chunk'
          ? 'export const value = 42;'
          : `export { value } from './m'; ${scenario === 'late-and-new-child' ? "export const next = () => import(/* webpackChunkName: 'next-child' */ './next-child');" : ''}`,
    'next-child.js': "export { value } from './m';",
    'child.js':
      scenario === 'inherited-growth'
        ? "export const load = () => import(/* webpackChunkName: 'grandchild' */ './grandchild');"
        : `export { value } from './m';
          ${scenario === 'late-addition-with-restore' ? "export { value as restored } from './x';" : ''}
          ${scenario === 'cycle' ? "export const back = () => import(/* webpackChunkName: 'parent' */ './parent-a');" : ''}`,
    'm.js': 'export const value = 42;',
    'extra.js':
      "export const load = () => import(/* webpackChunkName: 'child' */ './child');",
    'worker.js':
      scenario === 'shared-physical-chunk'
        ? 'export const value = 1;'
        : scenario === 'late-shared-physical-chunk'
          ? "export { value } from './m';"
          : "export const load = () => import(/* webpackChunkName: 'child' */ './child');",
    'worker-a.js':
      "export const load = () => import(/* webpackChunkName: 'worker-child' */ './worker-child');",
    'worker-b.js': `export { value } from './m';
      export const load = () => import(/* webpackChunkName: 'worker-next' */ './worker-next');`,
    'worker-child.js': "export { value } from './m';",
    'worker-next.js': "export { value } from './m';",
  };
  return defineConfig({
    mode: 'production',
    target: 'web',
    devtool: false,
    cache: false,
    incremental: false,
    entry:
      scenario === 'multiple-runtimes'
        ? { main: './index', extra: './extra' }
        : { main: './index' },
    output: {
      filename: `[name]-${index}.js`,
      chunkFilename: `[name]-${index}.js`,
    },
    optimization: {
      splitChunks: false,
      minimize: false,
      concatenateModules: false,
      usedExports: false,
      moduleIds: 'named',
      chunkIds: 'named',
    },
    plugins: [
      new experiments.VirtualModulesPlugin(modules),
      definePlugin({
        apply(compiler) {
          compiler.hooks.compilation.tap(
            'AssertAvailableModules',
            (compilation) => {
              compilation.hooks.afterOptimizeModules.tap(
                'AssertAvailableModules',
                () => {
                  const child = compilation.namedChunkGroups.get('child')!;
                  const count = child.chunks
                    .flatMap((c) => [
                      ...compilation.chunkGraph.getChunkModulesIterable(c),
                    ])
                    .filter((m) => /[\\/]m\.js$/.test(m.identifier())).length;
                  assert.equal(count, retains ? 1 : 0, scenario);
                  if (scenario === 'late-and-new-child') {
                    const next =
                      compilation.namedChunkGroups.get('next-child')!;
                    assert.ok(
                      next.chunks.every((chunk) =>
                        [
                          ...compilation.chunkGraph.getChunkModulesIterable(
                            chunk,
                          ),
                        ].every(
                          (module) => !/[\\/]m\.js$/.test(module.identifier()),
                        ),
                      ),
                    );
                  }
                  if (scenario === 'inherited-growth') {
                    const grandchild =
                      compilation.namedChunkGroups.get('grandchild')!;
                    assert.ok(
                      grandchild.chunks.every((chunk) =>
                        [
                          ...compilation.chunkGraph.getChunkModulesIterable(
                            chunk,
                          ),
                        ].every(
                          (module) => !/[\\/]m\.js$/.test(module.identifier()),
                        ),
                      ),
                    );
                  }
                  if (
                    scenario === 'shared-physical-chunk' ||
                    scenario === 'late-shared-physical-chunk'
                  ) {
                    assert.equal(
                      [...compilation.namedChunks.get('parent')!.groupsIterable]
                        .length,
                      2,
                    );
                  }
                  if (scenario === 'late-addition-with-restore') {
                    const parent = compilation.namedChunks.get('parent')!;
                    assert.ok(
                      [
                        ...compilation.chunkGraph.getChunkModulesIterable(
                          parent,
                        ),
                      ].some((module) =>
                        /[\\/]x\.js$/.test(module.identifier()),
                      ),
                      'restore the module lost from the inherited intersection',
                    );
                    assert.ok(
                      child.chunks.every((chunk) =>
                        [
                          ...compilation.chunkGraph.getChunkModulesIterable(
                            chunk,
                          ),
                        ].every(
                          (module) => !/[\\/]x\.js$/.test(module.identifier()),
                        ),
                      ),
                    );
                  }
                  if (scenario === 'late-worker-addition') {
                    const worker = compilation.namedChunks.get('worker')!;
                    assert.ok(
                      [
                        ...compilation.chunkGraph.getChunkModulesIterable(
                          worker,
                        ),
                      ].some((module) =>
                        /[\\/]m\.js$/.test(module.identifier()),
                      ),
                    );
                    for (const name of ['worker-child', 'worker-next']) {
                      const group = compilation.namedChunkGroups.get(name)!;
                      assert.ok(
                        group.chunks.every((chunk) =>
                          [
                            ...compilation.chunkGraph.getChunkModulesIterable(
                              chunk,
                            ),
                          ].every(
                            (module) =>
                              !/[\\/]m\.js$/.test(module.identifier()),
                          ),
                        ),
                        `${name} inherits the late worker module`,
                      );
                    }
                  }
                },
              );
            },
          );
        },
      }),
    ],
  });
});
