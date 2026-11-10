import { defineConfig, definePlugin } from '@rspack/cli';
import { container } from '@rspack/core';

let buildIndex = 0;

export default defineConfig({
  optimization: { minimize: false },
  plugins: [
    new container.ModuleFederationPlugin({
      name: 'remote_cache',
      remotes: {
        remote:
          "promise Promise.resolve().then(() => ({ get: () => Promise.resolve(() => 'remote value'), init: () => {} }))",
      },
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('CheckRemoteModuleCache', (stats) => {
          const compilation = stats.compilation;
          const externals = [...compilation.modules].filter((module) =>
            module.identifier().includes('remote value'),
          );
          expect(externals).toHaveLength(1);
          const chunks = [
            ...compilation.chunkGraph.getModuleChunksIterable(externals[0]),
          ];
          expect(chunks.map((chunk) => chunk.name)).toContain('main');
          if (buildIndex++ === 0) return;
          const info = stats.toJson({ all: false, cacheInfo: true }).cacheInfo![
            '|Compilation/modules'
          ];
          expect(info.misses).toBe(0);
          expect(info.reused).toBe(info.hits);
        });
      },
    }),
  ],
});
