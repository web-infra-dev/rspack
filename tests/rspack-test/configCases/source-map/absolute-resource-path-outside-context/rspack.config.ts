import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';

const libDir = path.join(import.meta.dirname, 'lib').replaceAll(path.sep, '/');
const queryModule = path
  .join(import.meta.dirname, 'app/query.js')
  .replaceAll(path.sep, '/');
const expectedSources = [
  [
    'export const windowsPath: number = 4;',
    String.raw`C:/repo/src/win.ts?variant=\raw#part`,
  ],
  ['module.exports = 42;', `${queryModule}?variant=a#part`],
  ['module.exports = 42;', `${queryModule}?variant=b#part`],
  ['export const n: number = 1;', `${libDir}/item.ts`],
  [
    'export const withQuery: number = 2;',
    `${libDir}/query.ts?path=a/../b#section`,
  ],
  [
    'export const withFragment: number = 3;',
    `${libDir}/fragment.ts#section/a/../b`,
  ],
];

interface SourceMap {
  sources: string[];
  sourcesContent: string[];
}

export default defineConfig(
  (['source-map', 'eval-source-map'] as const).map((devtool) => ({
    mode: 'development',
    context: path.join(import.meta.dirname, 'app'),
    devtool,
    module: {
      rules: [{ test: /item\.js$/, extractSourceMap: true }],
    },
    output: {
      devtoolModuleFilenameTemplate: '[absolute-resource-path]',
    },
    plugins: [
      definePlugin((compiler) => {
        compiler.hooks.afterEmit.tap('CheckSourcePaths', (compilation) => {
          const filename = compilation
            .getAssets()
            .find((asset) => asset.name.endsWith('.js'))!.name;
          const maps: SourceMap[] =
            devtool === 'source-map'
              ? [
                  JSON.parse(
                    compilation
                      .getAsset(`${filename}.map`)!
                      .source.source()
                      .toString(),
                  ),
                ]
              : Array.from(
                  compilation
                    .getAsset(filename)!
                    .source.source()
                    .toString()
                    .matchAll(
                      /sourceMappingURL=data:application\/json;charset=utf-8;base64,([\w+/=]+)/g,
                    ),
                  ([, base64]) =>
                    JSON.parse(Buffer.from(base64, 'base64').toString('utf8')),
                );
          for (const [content, file] of expectedSources) {
            const sources = maps.flatMap((map) =>
              map.sources.filter(
                (_, index) => map.sourcesContent[index]?.trimEnd() === content,
              ),
            );
            expect(sources).toContain(file);
          }
        });
      }),
    ],
  })),
);
