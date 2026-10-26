import { defineConfig, definePlugin } from '@rspack/cli';
import webpackSources from 'webpack-sources';

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
            const unicodeStart = content.indexOf('😀');
            if (unicodeStart < 0) throw new Error('Missing Unicode source');

            const { ConcatSource, ReplaceSource } = compiler.rspack.sources;
            const replaced = new ReplaceSource(asset.source);
            replaced.replace(start, start + placeholder.length - 1, 'updated');
            replaced.replace(unicodeStart, unicodeStart + 1, 'X');
            const updated = new ConcatSource(prefix, replaced);

            const fallbackReplaced = new webpackSources.ReplaceSource(
              asset.source,
            );
            fallbackReplaced.replace(
              start,
              start + placeholder.length - 1,
              'updated',
            );
            fallbackReplaced.replace(unicodeStart, unicodeStart + 1, 'X');
            const fallbackUpdated = new webpackSources.ConcatSource(
              prefix,
              fallbackReplaced,
            );

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
            compilation.emitAsset('fallback.js', fallbackUpdated);

            const nativeSource = compilation.getAsset('bundle0.js')?.source;
            const fallbackSource = compilation.getAsset('fallback.js')?.source;
            if (!nativeSource || !fallbackSource) {
              throw new Error('Missing structured source result');
            }
            expect(nativeSource.source()).toBe(fallbackSource.source());
            const nativeMap = nativeSource.map({ columns: true });
            const fallbackMap = fallbackSource.map({ columns: true });
            expect(nativeMap).toEqual(
              expect.objectContaining({
                version: fallbackMap?.version,
                names: fallbackMap?.names,
                sources: fallbackMap?.sources,
                sourcesContent: fallbackMap?.sourcesContent,
                mappings: fallbackMap?.mappings,
              }),
            );
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
