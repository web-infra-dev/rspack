import { defineConfig } from '@rspack/cli';
import {
  type Compiler,
  type Configuration,
  type OptimizationSplitChunksCacheGroup,
  rspack,
} from '@rspack/core';

const {
  Compilation,
  experiments: { VirtualModulesPlugin },
} = rspack;

const moduleCount = 160;
const entryNames = [
  'a',
  'b',
  'c',
  'd',
  ...Array.from({ length: 13 }, (_, index) => `entry-${index}`),
];
const observedOutputs = new Map<string, Map<string, Buffer>>();

function createVirtualModules() {
  const modules: Record<string, string> = {};
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
  label: string;

  constructor(label: string) {
    this.label = label;
  }

  apply(compiler: Compiler) {
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
              const nativeOutput = observedOutputs.get('native')!;
              const batchOutput = observedOutputs.get('batch')!;
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

function createConfig(
  label: string,
  name: OptimizationSplitChunksCacheGroup['name'],
): Configuration {
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

export default defineConfig([
  createConfig('native', false),
  createConfig('batch', (_module, chunks) => {
    expect(chunks.map((chunk) => chunk.name).sort()).toEqual(
      [...entryNames].sort(),
    );
    return undefined;
  }),
]);
