const {
  Compilation,
  experiments: { VirtualModulesPlugin },
} = require('@rspack/core');

const moduleCount = 160;
const entryNames = [
  'a',
  'b',
  'c',
  'd',
  ...Array.from({ length: 13 }, (_, index) => `entry-${index}`),
];
const observedOutputs = new Map();

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
  for (const entry of entryNames) {
    modules[`${entry}.js`] = `${imports}
it('loads entry ${entry}', () => {
  expect(${sum}).toBe(12720);
});`;
  }

  return modules;
}

class CompareChunkOutputPlugin {
  constructor(label) {
    this.label = label;
  }

  apply(compiler) {
    compiler.hooks.thisCompilation.tap(
      'CompareChunkOutputPlugin',
      (compilation) => {
        compilation.hooks.processAssets.tap(
          {
            name: 'CompareChunkOutputPlugin',
            stage: Compilation.PROCESS_ASSETS_STAGE_SUMMARIZE,
          },
          () => {
            const output = new Map(
              Object.entries(compilation.assets).map(([name, source]) => [
                name,
                Buffer.from(source.source()),
              ]),
            );
            observedOutputs.set(this.label, output);

            if (observedOutputs.size === 2) {
              const nativeOutput = observedOutputs.get('native');
              const batchOutput = observedOutputs.get('batch');
              expect([...batchOutput.keys()].sort()).toEqual(
                [...nativeOutput.keys()].sort(),
              );
              for (const [name, source] of batchOutput) {
                expect(source).toEqual(nativeOutput.get(name));
              }
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
    entry: Object.fromEntries(entryNames.map((entry) => [entry, `./${entry}`])),
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
            filename: 'shared.js',
            name,
          },
        },
      },
    },
    plugins: [
      new VirtualModulesPlugin(createVirtualModules()),
      new CompareChunkOutputPlugin(label),
    ],
  };
}

/** @type {import('@rspack/core').Configuration[]} */
module.exports = [
  createConfig('native', false),
  createConfig('batch', (_module, chunks) => {
    for (const chunk of chunks) {
      void chunk.name;
    }
    return undefined;
  }),
];
