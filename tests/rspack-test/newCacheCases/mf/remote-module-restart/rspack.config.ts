import { defineConfig, definePlugin } from '@rspack/cli';
import { container } from '@rspack/core';

let buildIndex = 0;

export default defineConfig({
  optimization: { minimize: false },
  plugins: [
    new container.ModuleFederationPluginV1({
      name: 'remote_cache_v1',
      remotes: {
        remote:
          "promise Promise.resolve().then(() => ({ get: () => Promise.resolve(() => 'remote value'), init: () => {} }))",
      },
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('CheckRemoteModuleCache', (stats) => {
          const remoteModules = [...stats.compilation.modules].filter(
            (module) => module.identifier().startsWith('remote ('),
          );
          expect(remoteModules).toHaveLength(1);
          if (buildIndex++ === 0) return;
          const info = stats.toJson({ all: false, cacheInfo: true }).cacheInfo![
            '|Compilation/modules'
          ];
          expect(info.hits).toBe(3);
          expect(info.misses).toBe(0);
          expect(info.reused).toBe(3);
        });
      },
    }),
  ],
});
