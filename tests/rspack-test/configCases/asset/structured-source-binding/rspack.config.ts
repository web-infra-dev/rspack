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
            const original = asset.source;
            let replaced = new ReplaceSource(original);
            replaced.replace(
              start,
              start + placeholder.length - 1,
              '😀intermediate',
            );
            replaced.replace(unicodeStart, unicodeStart + 1, 'X');
            const layers = [original, replaced];

            let fallbackReplaced = new webpackSources.ReplaceSource(
              asset.source,
            );
            fallbackReplaced.replace(
              start,
              start + placeholder.length - 1,
              '😀intermediate',
            );
            fallbackReplaced.replace(unicodeStart, unicodeStart + 1, 'X');

            // Each layer uses positions in the previous layer's output, including
            // edits to Unicode text introduced by an inner replacement.
            for (const [before, after, concat] of [
              ['intermediate', 'first', false],
              ['😀first', '😀临时', true],
              ['😀临时', 'updated', true],
            ] as const) {
              const combined = concat
                ? new ConcatSource(
                    '/* 😀 nested */\n',
                    replaced,
                    '\n/* nested */',
                  )
                : replaced;
              if (concat) layers.push(combined);
              const fallbackCombined = concat
                ? new webpackSources.ConcatSource(
                    '/* 😀 nested */\n',
                    fallbackReplaced,
                    '\n/* nested */',
                  )
                : fallbackReplaced;
              const position = fallbackCombined
                .source()
                .toString()
                .indexOf(before);
              if (position < 0)
                throw new Error(`Missing nested replacement: ${before}`);
              replaced = new ReplaceSource(combined);
              replaced.replace(position, position + before.length - 1, after);
              layers.push(replaced);
              fallbackReplaced = new webpackSources.ReplaceSource(
                fallbackCombined,
              );
              fallbackReplaced.replace(
                position,
                position + before.length - 1,
                after,
              );
            }
            const updated = new ConcatSource(prefix, replaced);
            const fallbackUpdated = new webpackSources.ConcatSource(
              prefix,
              fallbackReplaced,
            );

            // Every layer must stay native: neither text nor map serialization
            // may run on the JS graph, even for an intermediate ReplaceSource.
            for (const source of [...layers, updated]) {
              for (const method of ['source', 'map']) {
                Object.defineProperty(source, method, {
                  value: () => {
                    throw new Error(
                      `Nested source used the fallback ${method}()`,
                    );
                  },
                });
              }
            }
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
