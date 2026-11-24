import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import type { Configuration } from '@rspack/core';
import { fixtures } from './fixtures.mjs';

const customPath = String.raw`custom://host/source\name.js`;
const customRequest = `${customPath}${String.raw`?option=\value`}`;

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
            rules: [
              {
                test: /module\.js$/,
                use: ['./loader.mjs'],
                layer: String.raw`layer\value`,
              },
              {
                test: /^custom:/,
                type: 'javascript/auto',
                use: ['./loader.mjs'],
              },
            ],
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
                        ? 'fields:///[resource]:::[resource-path]:::[identifier]:::[short-identifier]:::[all-loaders]:::[loaders]:::[all-loaders][resource]:::[loaders][resource]:::[relative-resource-path]'
                        : '[absolute-resource-path]',
            devtoolFallbackModuleFilenameTemplate:
              variant === 'fallback-callback'
                ? (info) => `fallback:///${info.absoluteResourcePath}`
                : 'fallback:///[absolute-resource-path]?[hash]',
          },
          plugins: [
            definePlugin({
              apply(compiler) {
                compiler.hooks.normalModuleFactory.tap(
                  'ResolveCustomSource',
                  (normalModuleFactory) => {
                    normalModuleFactory.hooks.resolveForScheme
                      .for('custom')
                      .tap('ResolveCustomSource', (resource) => {
                        expect(resource.path).toBe(customPath);
                        return true;
                      });
                  },
                );
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
                  let checkedCustom = false;
                  for (const map of maps) {
                    map.sourcesContent.forEach((content, index) => {
                      if (content === 'custom-source') {
                        checkedCustom = true;
                        const actual = map.sources[index];
                        if (variant === 'absolute') {
                          expect(actual).toBe(customPath);
                        } else if (variant === 'fields') {
                          const fields = actual
                            .slice('fields:///'.length)
                            .split(':::');
                          expect(fields[0]).toBe(customRequest);
                          expect(fields[1]).toBe(customPath);
                          expect(fields[8]).toBe(customPath);
                        }
                      }
                      if (content === 'module-identifier') {
                        const actual = map.sources[index];
                        if (variant === 'fields') {
                          // Existing contextify behavior normalizes the layer separator on Windows.
                          expect(
                            actual.slice('fields:///'.length).split(':::')[2],
                          ).toContain(
                            process.platform === 'win32'
                              ? '|layer/value'
                              : String.raw`|layer\value`,
                          );
                        } else if (variant === 'absolute') {
                          expect(actual).toBe(
                            path
                              .join(import.meta.dirname, 'module.js')
                              .replaceAll('\\', '/'),
                          );
                        }
                      }
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
                          .split(':::');
                        if (fixture.source.startsWith('builtin:')) {
                          expect(fields[4]).toBe(
                            String.raw`builtin:swc-loader?option=\loader\value`,
                          );
                        }
                        if (fixture.source.includes('|sync|')) {
                          const pathFields = [
                            fields[0],
                            fields[1],
                            fields[2],
                            fields[3],
                            fields[6],
                            fields[7],
                          ];
                          for (const field of pathFields) {
                            expect(field).toBe(fixture.source);
                          }
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
                        const source = fixture.opaque
                          ? raw
                          : fixture.normalized;
                        const expected =
                          variant === 'literal'
                            ? `literal://prefix\\value/${source}`
                            : variant === 'fallback'
                              ? `fallback:///${source}`
                              : source;
                        expect(
                          variant === 'fallback'
                            ? actual.replace(/\?[a-f0-9]{4}$/, '')
                            : actual,
                        ).toBe(expected);
                      }
                    });
                  }
                  expect(checked).toBeGreaterThanOrEqual(fixtures.length - 1);
                  expect(checkedCustom).toBe(true);
                });
              },
            }),
          ],
        })),
  ),
);
