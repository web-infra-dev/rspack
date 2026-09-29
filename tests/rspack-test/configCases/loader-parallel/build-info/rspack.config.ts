import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

class Plugin {
  apply(compiler: Compiler) {
    const { NormalModule } = compiler.rspack;

    compiler.hooks.finishMake.tap('PLUGIN', (compilation) => {
      const module = [...compilation.modules].find(
        (m) => m instanceof NormalModule && m.resource?.endsWith('lib.js'),
      )!;
      const { buildInfo } = module;

      expect(buildInfo.parallel).toBe(true);

      expect(buildInfo.trail).toEqual(['main', 'worker']);

      expect(buildInfo.buildDependencies.has('./build.txt')).toBe(true);
    });
  }
}

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /lib\.js/,
        use: [
          { loader: './worker-loader.mjs', parallel: true, options: {} },
          { loader: './seed-loader.mjs' },
        ],
      },
    ],
  },
  plugins: [new Plugin()],
});
