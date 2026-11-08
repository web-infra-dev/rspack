import { defineConfig } from '@rspack/cli';
import { type Compiler } from '@rspack/core';

class Plugin {
  expectedModuleIds: (string | number)[];

  constructor(expectedModuleIds: (string | number)[]) {
    this.expectedModuleIds = expectedModuleIds;
  }

  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.processAssets.tap('Test', () => {
        const moduleIds = Array.from(compilation.modules).map((m) =>
          compilation.chunkGraph.getModuleId(m),
        );
        expect(moduleIds).toEqual(this.expectedModuleIds);
      });
    });
  }
}

export default defineConfig([
  {
    optimization: {
      moduleIds: 'named',
    },
    plugins: [new Plugin(['./index.js'])],
  },
  {
    optimization: {
      moduleIds: 'natural',
    },
    plugins: [new Plugin([0])],
  },
  {
    optimization: {
      moduleIds: 'deterministic',
    },
    plugins: [new Plugin([237])],
  },
]);
