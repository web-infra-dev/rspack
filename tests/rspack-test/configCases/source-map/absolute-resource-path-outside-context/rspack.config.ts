import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';

const expectedSources = [
  [
    'export const n: number = 1;',
    path.join(import.meta.dirname, 'lib/item.ts').replaceAll('\\', '/'),
  ],
  [
    'export const withQuery: number = 2;',
    path.join(import.meta.dirname, 'lib/query.ts').replaceAll('\\', '/') +
      '?path=a/../b#section',
  ],
  [
    'export const withFragment: number = 3;',
    path.join(import.meta.dirname, 'lib/fragment.ts').replaceAll('\\', '/') +
      '#section/a/../b',
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
            const map = maps.find((map) =>
              map.sourcesContent.includes(content),
            );
            expect(map?.sources[map.sourcesContent.indexOf(content)]).toBe(
              file,
            );
          }
        });
      }),
    ],
  })),
);
