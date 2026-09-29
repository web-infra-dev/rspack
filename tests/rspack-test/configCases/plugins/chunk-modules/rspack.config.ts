import { defineConfig, definePlugin } from '@rspack/cli';
import { type Module, NormalModule } from '@rspack/core';

export default defineConfig({
  plugins: [
    definePlugin(function plugin(compiler) {
      function moduleStringify(module: Module) {
        return {
          resource:
            module instanceof NormalModule && module.resource
              ? normalizePathLike(module.resource)
              : undefined,
          context: module.context
            ? normalizePathLike(module.context)
            : undefined,
          identifier: normalizePathLike(module.identifier()),
        };
      }

      function normalizePathLike(value: string) {
        return value
          .replace(compiler.context, '<COMPILER_CONTEXT>')
          .replace(/\\/g, '/');
      }

      function compareByIdentifier(a: Module, b: Module) {
        return a.identifier().localeCompare(b.identifier());
      }

      compiler.hooks.compilation.tap('plugin', (compilation) => {
        compilation.hooks.processAssets.tap('plugin', () => {
          const chunkModules: Record<
            string,
            {
              modules: ReturnType<typeof moduleStringify>[];
              entryModules: ReturnType<typeof moduleStringify>[];
            }
          > = {};
          for (let chunk of compilation.chunks) {
            const modules = [
              ...compilation.chunkGraph.getChunkModulesIterable(chunk),
            ].sort(compareByIdentifier);
            const entryModules = [
              ...compilation.chunkGraph.getChunkEntryModulesIterable(chunk),
            ].sort(compareByIdentifier);
            chunkModules[chunk.id!] = {
              modules: modules.map(moduleStringify),
              entryModules: entryModules.map(moduleStringify),
            };
          }
          compilation.emitAsset(
            'data.json',
            new compiler.rspack.sources.RawSource(
              JSON.stringify(chunkModules, null, 2),
            ),
          );
        });
      });
    }),
  ],
});
