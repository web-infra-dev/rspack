import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig(
  (['source-map', 'hidden-source-map', 'inline-source-map'] as const).map(
    (devtool) => ({
      target: 'web',
      devtool,
      plugins: [
        definePlugin((compiler) => {
          let filename: string;
          let content: string;
          let originalMap: unknown;
          compiler.hooks.compilation.tap('test', (compilation) => {
            compilation.hooks.afterProcessAssets.tap('test', () => {
              filename = compilation
                .getAssets()
                .find((asset) => asset.name.endsWith('.js'))!.name;
              content = compilation
                .getAsset(filename)!
                .source.source()
                .toString();
              expect(compilation.getAsset(filename)!.source.map()).toBeNull();
              const module = [...compilation.modules].find((module) =>
                module.identifier().endsWith('index.js'),
              )!;
              originalMap = module.originalSource()!.map();
              expect(originalMap).toBeTruthy();
            });
          });
          compiler.hooks.afterEmit.tap('test', (compilation) => {
            expect(compilation.getAsset(filename)!.source.map()).toBeNull();
            expect(
              compilation.getAsset(filename)!.source.source().toString(),
            ).toBe(content);
            const module = [...compilation.modules].find((module) =>
              module.identifier().endsWith('index.js'),
            )!;
            expect(module.originalSource()!.map()).toEqual(originalMap);
            if (devtool !== 'inline-source-map') {
              const map = compilation.getAsset(`${filename}.map`)!;
              expect(
                JSON.parse(map.source.source().toString()).mappings.length,
              ).toBeGreaterThan(0);
            }
          });
        }),
      ],
    }),
  ),
);
