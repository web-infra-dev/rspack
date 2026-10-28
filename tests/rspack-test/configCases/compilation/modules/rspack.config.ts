import { defineConfig } from '@rspack/cli';
import { type Compiler, NormalModule, rspack } from '@rspack/core';

const PLUGIN_NAME = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    const { AsyncDependenciesBlock } = compiler.rspack;

    compiler.hooks.make.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: PLUGIN_NAME,
          stage: rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
        },
        () => {
          const module = Array.from(compilation.modules).find(
            (module) =>
              module instanceof NormalModule &&
              module.rawRequest === './index.js',
          )!;
          const block = module.blocks[0];
          expect(block instanceof AsyncDependenciesBlock).toBe(true);
          expect(block.constructor.name).toBe('AsyncDependenciesBlock');

          const dependency = module.dependencies[0];
          expect(block.dependencies[0].request).toBe('./a');
          expect(dependency.request).toBe('./b');
        },
      );
    });
  }
}

export default defineConfig({
  entry: './index.js',
  plugins: [new Plugin()],
});
