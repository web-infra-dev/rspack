const {
  Compilation,
  experiments: { VirtualModulesPlugin },
} = require('@rspack/core');

const moduleCount = 160;
const observedModuleOrders = new Map();

function createVirtualModules() {
  const modules = {};
  for (let index = 0; index < moduleCount; index++) {
    modules[`shared-${index}.js`] = `module.exports = ${index};`;
  }

  const imports = Array.from(
    { length: moduleCount },
    (_, index) => `const value${index} = require('./shared-${index}');`,
  ).join('\n');
  const sum = Array.from(
    { length: moduleCount },
    (_, index) => `value${index}`,
  ).join(' + ');
  for (const entry of ['a', 'b', 'c', 'd']) {
    modules[`${entry}.js`] = `${imports}
it('loads entry ${entry}', () => {
  expect(${sum}).toBe(12720);
});`;
  }

  return modules;
}

class CompareChunkModuleOrderPlugin {
  constructor(label) {
    this.label = label;
  }

  apply(compiler) {
    compiler.hooks.thisCompilation.tap(
      'CompareChunkModuleOrderPlugin',
      (compilation) => {
        compilation.hooks.processAssets.tap(
          {
            name: 'CompareChunkModuleOrderPlugin',
            stage: Compilation.PROCESS_ASSETS_STAGE_SUMMARIZE,
          },
          () => {
            const sharedChunk = [...compilation.chunks].find(
              (chunk) => chunk.name === 'shared',
            );
            expect(sharedChunk).toBeTruthy();
            const moduleOrder = compilation.chunkGraph
              .getChunkModules(sharedChunk)
              .map((module) => module.identifier());
            expect(moduleOrder).toHaveLength(moduleCount);
            observedModuleOrders.set(this.label, moduleOrder);

            if (observedModuleOrders.size === 2) {
              expect(observedModuleOrders.get('batch')).toEqual(
                observedModuleOrders.get('native'),
              );
            }
          },
        );
      },
    );
  }
}

function createConfig(label, name) {
  return {
    name: label,
    mode: 'development',
    target: 'node',
    entry: {
      a: './a',
      b: './b',
      c: './c',
      d: './d',
    },
    output: {
      filename: '[name].js',
    },
    optimization: {
      concatenateModules: false,
      splitChunks: {
        chunks: 'all',
        minSize: 0,
        cacheGroups: {
          shared: {
            test: /shared-\d+\.js$/,
            minChunks: 2,
            name,
          },
        },
      },
    },
    plugins: [
      new VirtualModulesPlugin(createVirtualModules()),
      new CompareChunkModuleOrderPlugin(label),
    ],
  };
}

/** @type {import('@rspack/core').Configuration[]} */
module.exports = [
  createConfig('native', 'shared'),
  createConfig('batch', () => 'shared'),
];
