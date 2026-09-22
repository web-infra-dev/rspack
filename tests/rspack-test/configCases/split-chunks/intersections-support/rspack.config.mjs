import assert from 'node:assert/strict';
import { experiments } from '@rspack/core';

const { VirtualModulesPlugin } = experiments;

const moduleCount = 130;
const indices = Array.from({ length: moduleCount }, (_, i) => i);
const modules = Object.fromEntries(
  indices.map((i) => [
    `m${i}.js`,
    `export default "module-${String(i).padStart(3, '0')}";`,
  ]),
);
const entry = {};
function addEntry(name, selected) {
  entry[name] = `./${name}`;
  modules[`${name}.js`] = `${selected
    .map((i) => `import m${i} from './m${i}';`)
    .join('\n')}
it('loads ${name}', () => {
  expect([${selected.map((i) => `m${i}`).join(',')}]).toEqual(${JSON.stringify(
    selected.map((i) => `module-${String(i).padStart(3, '0')}`),
  )});
});`;
}
addEntry('a', indices);
addEntry('b', indices);
for (let i = 0; i < moduleCount; i += 2) {
  addEntry(`p${i}`, [i, i + 1]);
  addEntry(`q${i}`, [i, i + 1]);
  addEntry(`u${i}`, [i]);
  addEntry(`u${i + 1}`, [i + 1]);
}

// Every module has a distinct original chunk set. The a/b intersection has
// dense support; each p/q pair is supported by only two of the 130 sets.
/** @type {import('@rspack/core').Configuration[]} */
export default [false, true]
  .flatMap((usedExports) =>
    ['all', 'pairs', 'common'].map((selection) => ({ usedExports, selection })),
  )
  .map(({ usedExports, selection }, index) => ({
    mode: 'production',
    target: 'node',
    entry,
    output: {
      filename: `[name]-${index}.js`,
      chunkFilename: `[name]-${index}.js`,
    },
    optimization: {
      minimize: false,
      concatenateModules: false,
      splitChunks: {
        dedupDepth: 1,
        usedExports,
        chunks:
          selection === 'all'
            ? 'all'
            : (chunk) =>
                (chunk.name === 'a' || chunk.name === 'b') ===
                (selection === 'common'),
        minChunks: 2,
        minSize: modules['m0.js'].length * 2,
        maxInitialRequests: Infinity,
        maxAsyncRequests: Infinity,
        cacheGroups: {
          default: false,
          defaultVendors: false,
          shared: { test: /[\\/]m\d+\.js$/ },
        },
      },
    },
    plugins: [
      new VirtualModulesPlugin(modules),
      {
        apply(compiler) {
          compiler.hooks.done.tap('AssertIntersectionSupport', (stats) => {
            const { modules } = stats.toJson({
              all: false,
              modules: true,
              ids: true,
              groupModulesByType: false,
              groupModulesByPath: false,
            });
            const shared = modules.filter((module) =>
              /^\.\/m\d+\.js$/.test(module.name),
            );
            assert.equal(shared.length, moduleCount);
            const common = shared[0].chunks.filter((chunk) =>
              shared.every((module) => module.chunks.includes(chunk)),
            );
            // With all chunks eligible, the four-chunk pair candidates win
            // before the two-chunk common candidate. Filtering to a/b exercises
            // the dense support; filtering them out exercises sparse support.
            assert.equal(
              common.length,
              selection === 'all' ? 0 : selection === 'pairs' ? 2 : 1,
            );
            for (const module of shared) {
              assert.equal(module.chunks.length, selection === 'all' ? 2 : 4);
            }
            for (let i = 0; i < moduleCount; i += 2) {
              const left = shared.find(
                (module) => module.name === `./m${i}.js`,
              );
              const right = shared.find(
                (module) => module.name === `./m${i + 1}.js`,
              );
              assert.equal(
                left.chunks.filter((chunk) => right.chunks.includes(chunk))
                  .length,
                selection === 'all' ? 1 : 3,
              );
            }
          });
        },
      },
    ],
  }));
