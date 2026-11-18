import { defineConfig, definePlugin } from '@rspack/cli';
import type { Configuration } from '@rspack/core';
import { fixtures } from './fixtures.mjs';

const variants = [
  'absolute',
  'literal',
  'callback',
  'fallback',
  'fallback-callback',
  'fields',
  'default',
] as const;

export default defineConfig(
  (['source-map', 'inline-source-map', 'eval-source-map'] as const).flatMap(
    (devtool) =>
      variants
        .filter(
          (variant) =>
            devtool !== 'eval-source-map' || !variant.startsWith('fallback'),
        )
        .map((variant): Configuration => ({
          mode: 'development',
          devtool,
          module: {
            rules: [{ test: /module\.js$/, use: ['./loader.mjs'] }],
          },
          output: {
            filename: 'main.js',
            devtoolNamespace: 'namespace\\value',
            devtoolModuleFilenameTemplate:
              variant === 'default'
                ? undefined
                : variant === 'callback'
                  ? (info) => `callback:///${info.absoluteResourcePath}`
                  : variant.startsWith('fallback')
                    ? 'primary://same'
                    : variant === 'literal'
                      ? 'literal://prefix\\value/[absolute-resource-path]'
                      : variant === 'fields'
                        ? 'fields:///[resource]|[resource-path]|[identifier]|[short-identifier]|[all-loaders]|[loaders]|[all-loaders][resource]|[loaders][resource]|[relative-resource-path]'
                        : '[absolute-resource-path]',
            devtoolFallbackModuleFilenameTemplate:
              variant === 'fallback-callback'
                ? (info) => `fallback:///${info.absoluteResourcePath}`
                : 'fallback:///[absolute-resource-path]?[hash]',
          },
          plugins: [
            definePlugin({
              apply(compiler) {
                compiler.hooks.done.tap('CheckTemplatePaths', (stats) => {
                  const compilation = stats.compilation;
                  const maps: {
                    sources: string[];
                    sourcesContent: (string | null)[];
                  }[] = [];
                  if (devtool === 'source-map') {
                    maps.push(
                      JSON.parse(
                        compilation
                          .getAsset('main.js.map')!
                          .source.source()
                          .toString(),
                      ),
                    );
                  } else {
                    const code = compilation
                      .getAsset('main.js')!
                      .source.source()
                      .toString();
                    for (const match of code.matchAll(
                      /sourceMappingURL=data:application\/json;charset=utf-8;base64,([\w+/=]+)/g,
                    )) {
                      maps.push(
                        JSON.parse(Buffer.from(match[1], 'base64').toString()),
                      );
                    }
                  }
                  expect(maps.length).toBeGreaterThan(0);
                  let checked = 0;
                  for (const map of maps) {
                    map.sourcesContent.forEach((content, index) => {
                      if (!content?.startsWith('fixture:')) return;
                      const fixture =
                        fixtures[Number(content.slice('fixture:'.length))];
                      const actual = map.sources[index];
                      if (
                        variant.startsWith('fallback') &&
                        actual === 'primary://same'
                      )
                        return;
                      checked++;
                      const raw = fixture.absolute ?? fixture.source;
                      if (
                        variant === 'callback' ||
                        variant === 'fallback-callback'
                      ) {
                        expect(actual).toBe(
                          `${variant === 'callback' ? 'callback' : 'fallback'}:///${raw}`,
                        );
                      } else if (variant === 'fields') {
                        const fields = actual
                          .slice('fields:///'.length)
                          .split('|');
                        if (fixture.source.startsWith('builtin:')) {
                          expect(fields[4]).toBe(
                            String.raw`builtin:swc-loader?option=\loader\value`,
                          );
                          expect(fields[2]).not.toContain(
                            String.raw`Z:\project`,
                          );
                        }
                        if (fixture.source === fixtures[0].source) {
                          for (const field of [
                            fields[0],
                            fields[2],
                            fields[3],
                            fields[6],
                            fields[7],
                          ]) {
                            expect(field).toContain(
                              String.raw`?query=C:\query\value#fragment\value`,
                            );
                          }
                          if (devtool !== 'eval-source-map') {
                            expect(fields[8]).toContain(
                              String.raw`?query=C:\query\value#fragment\value`,
                            );
                          }
                          expect(fields[1]).not.toContain('\\');
                        }
                      } else if (variant === 'default') {
                        expect(actual).toMatch(
                          /^(webpack|rspack):\/\/namespace\\value\//,
                        );
                        if (fixture.source === fixtures[0].source) {
                          expect(actual).toMatch(
                            /project\/src\/index\.ts(?:\?[a-f0-9]{4})?$/,
                          );
                        }
                      } else {
                        const expected =
                          variant === 'literal'
                            ? `literal://prefix\\value/${fixture.normalized}`
                            : variant === 'fallback'
                              ? `fallback:///${fixture.normalized}`
                              : fixture.normalized;
                        expect(
                          variant === 'fallback'
                            ? actual.replace(/\?[a-f0-9]{4}$/, '')
                            : actual,
                        ).toBe(expected);
                      }
                    });
                  }
                  expect(checked).toBeGreaterThanOrEqual(fixtures.length - 1);
                });
              },
            }),
          ],
        })),
  ),
);
