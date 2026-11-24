import { defineConfig, definePlugin } from '@rspack/cli';

const devtools = [
  'source-map',
  'inline-source-map',
  'eval-source-map',
] as const;
const templates = [
  '[absolute-resource-path]',
  '[relative-resource-path]',
  'webpack://[namespace]/[relative-resource-path]',
] as const;
const identifiers = [
  String.raw`./foo|sync|/^\.\/.*\.js$/`,
  String.raw`./foo|sync|/^\.\/(?:a%7Cb)\.js$/`,
];

interface SourceMap {
  sources: string[];
  sourcesContent: string[];
}

export default defineConfig(
  devtools.flatMap((devtool) =>
    templates.map((template) => ({
      mode: 'development',
      entry: './index.js',
      devtool,
      output: {
        devtoolNamespace: '',
        devtoolModuleFilenameTemplate: template,
      },
      plugins: [
        definePlugin((compiler) => {
          compiler.hooks.afterEmit.tap(
            'CheckContextSourceNames',
            (compilation) => {
              const jsAsset = compilation
                .getAssets()
                .find((asset) => asset.name.endsWith('.js'))!;
              const maps: SourceMap[] =
                devtool === 'source-map'
                  ? [
                      JSON.parse(
                        compilation
                          .getAsset(`${jsAsset.name}.map`)!
                          .source.source()
                          .toString(),
                      ),
                    ]
                  : Array.from(
                      jsAsset.source
                        .source()
                        .toString()
                        .matchAll(
                          /sourceMappingURL=data:application\/json;charset=utf-8;base64,([\w+/=]+)/g,
                        ),
                      ([, base64]) =>
                        JSON.parse(
                          Buffer.from(base64, 'base64').toString('utf8'),
                        ),
                    );
              const scheme = template.startsWith('webpack://')
                ? 'webpack'
                : (compiler.options.experiments?.runtimeMode ?? 'webpack');
              const virtualSources = maps.flatMap((map) =>
                map.sources.filter((_, index) =>
                  map.sourcesContent[index]?.includes(
                    'function __rspack_context_module(req)',
                  ),
                ),
              );
              expect(virtualSources.sort()).toEqual(
                identifiers
                  .map((identifier) => `${scheme}:///${identifier}`)
                  .sort(),
              );
            },
          );
        }),
      ],
    })),
  ),
);
