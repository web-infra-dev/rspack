import { defineConfig, definePlugin } from '@rspack/cli';

const prefix = 'globalThis.__STRUCTURED_SOURCE_PREFIX__ = true;\n';
const placeholder = '__REPLACE_ME__';

export default defineConfig({
  mode: 'production',
  target: 'web',
  devtool: 'source-map',
  optimization: {
    minimize: false,
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.thisCompilation.tap('StructuredSource', (compilation) => {
        compilation.hooks.processAssets.tap(
          {
            name: 'StructuredSource',
            stage: compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
          },
          () => {
            const asset = compilation.getAsset('bundle0.js');
            if (!asset) throw new Error('Missing bundle0.js');

            const content = asset.source.source();
            if (typeof content !== 'string') {
              throw new Error('Expected bundle0.js to be text');
            }
            const start = content.indexOf(placeholder);
            if (start < 0) throw new Error('Missing source placeholder');

            const { ConcatSource, ReplaceSource } = compiler.rspack.sources;
            const replaced = new ReplaceSource(asset.source);
            replaced.replace(start, start + placeholder.length - 1, 'updated');
            const updated = new ConcatSource(prefix, replaced);

            // Falling back to full source serialization would call map() here.
            Object.defineProperty(replaced, 'map', {
              value: () => {
                throw new Error('ReplaceSource used the fallback path');
              },
            });
            Object.defineProperty(updated, 'map', {
              value: () => {
                throw new Error('ConcatSource used the fallback path');
              },
            });
            compilation.updateAsset('bundle0.js', updated);
          },
        );

        compilation.hooks.processAssets.tap(
          {
            name: 'StructuredSourceVerify',
            stage: compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_REPORT,
          },
          () => {
            const asset = compilation.getAsset('bundle0.js');
            if (!asset) throw new Error('Missing updated bundle0.js');
            const content = asset.source.source();
            expect(typeof content).toBe('string');
            if (typeof content !== 'string') return;
            expect(content.startsWith(prefix)).toBe(true);
            expect(content).not.toContain(placeholder);

            const sourceMapName = asset.info.related?.sourceMap;
            expect(typeof sourceMapName).toBe('string');
            const sourceMapAsset = compilation.getAsset(
              sourceMapName as string,
            );
            if (!sourceMapAsset) throw new Error('Missing bundle source map');
            const sourceMap = JSON.parse(
              sourceMapAsset.source.source().toString(),
            );
            expect(sourceMap.mappings.length).toBeGreaterThan(0);
            expect(sourceMap.sourcesContent.join('\n')).toContain(placeholder);
          },
        );
      });
    }),
  ],
});
