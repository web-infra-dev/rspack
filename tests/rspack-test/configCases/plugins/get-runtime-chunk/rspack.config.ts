import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

const table: Record<string, string> = {
  ['main1']: 'main1',
  ['main2']: 'main2-runtime',
};

function plugin(compiler: Compiler) {
  compiler.hooks.compilation.tap('plugin', (compilation) => {
    compilation.hooks.processAssets.tap('plugin', () => {
      for (let [name, entrypoint] of compilation.entrypoints.entries()) {
        const runtimeChunk = entrypoint.getRuntimeChunk();
        expect(runtimeChunk!.name).toBe(table[name]);
      }
    });
  });
}

const common = defineConfig({
  output: {
    filename: '[name].js',
  },
  plugins: [plugin],
});

export default defineConfig([
  {
    ...common,
    entry: {
      main1: './entry1.js',
    },
  },
  {
    ...common,
    entry: {
      main2: {
        import: './entry2.js',
        runtime: 'main2-runtime',
      },
    },
  },
]);
