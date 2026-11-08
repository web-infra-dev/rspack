import { defineConfig } from '@rspack/cli';
import {
  type Compiler,
  type ConcatenatedModule,
  type NormalModule,
} from '@rspack/core';

const PLUGIN_NAME = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.make.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.processAssets.tap(PLUGIN_NAME, () => {
        const entrypoint = Array.from(compilation.entrypoints.values())[0];
        const entrypointChunk = entrypoint.chunks[0];
        const entrypointModule = compilation.chunkGraph.getChunkModules(
          entrypointChunk,
        )[0] as ConcatenatedModule;
        expect((entrypointModule.modules[0] as NormalModule).rawRequest).toBe(
          './foo',
        );
        expect((entrypointModule.modules[1] as NormalModule).rawRequest).toBe(
          './index.js',
        );
      });
    });
  }
}

export default defineConfig({
  entry: './index.js',
  plugins: [new Plugin()],
  optimization: {
    concatenateModules: true,
  },
});
