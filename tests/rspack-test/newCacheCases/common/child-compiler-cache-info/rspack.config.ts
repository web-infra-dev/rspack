import { defineConfig, definePlugin } from '@rspack/cli';

const PLUGIN_NAME = 'ChildCompilerCacheInfoTest';
let buildIndex = 0;

export default defineConfig({
  name: 'root',
  mode: 'none',
  incremental: false,
  experiments: {
    newCache: { resolver: false },
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.finishMake.tapPromise(
          PLUGIN_NAME,
          async (compilation) => {
            if (compilation.compiler !== compiler) return;

            let cacheInfo =
              compilation.getStats().toJson({
                all: false,
                cacheInfo: true,
              }).cacheInfo ?? {};
            const childCount = buildIndex === 0 ? 2 : 1;
            for (let index = 0; index < childCount; index++) {
              const child = compilation.createChildCompiler(
                'child',
                { filename: `child-${index}.js` },
                [
                  new compiler.rspack.EntryPlugin(
                    compiler.context,
                    './child.js',
                    {
                      name: 'main',
                    },
                  ),
                ],
              );
              await new Promise<void>((resolve, reject) => {
                child.runAsChild((error) =>
                  error ? reject(error) : resolve(),
                );
              });

              const previous = cacheInfo;
              cacheInfo = compilation.getStats().toJson({
                all: false,
                cacheInfo: true,
              }).cacheInfo!;
              expect(cacheInfo).toMatchObject(previous);
            }
          },
        );

        compiler.hooks.done.tap(PLUGIN_NAME, (stats) => {
          const cacheInfo = stats.toJson({
            all: false,
            cacheInfo: true,
          }).cacheInfo!;
          const prefix = compiler.compilerPath;
          expect(cacheInfo[`${prefix}|child|0|Compilation/modules`]).toEqual(
            buildIndex === 0
              ? { hits: 0, misses: 1, reused: 0, stores: 1 }
              : { hits: 1, misses: 0, reused: 1, stores: 0 },
          );
          if (buildIndex === 0) {
            expect(cacheInfo[`${prefix}|Compilation/modules`]).toEqual({
              hits: 0,
              misses: 2,
              reused: 0,
              stores: 2,
            });
            expect(cacheInfo[`${prefix}|child|1|Compilation/modules`]).toEqual({
              hits: 0,
              misses: 1,
              reused: 0,
              stores: 1,
            });
          } else {
            expect(
              Object.keys(cacheInfo).some((name) =>
                name.startsWith(`${prefix}|child|1|`),
              ),
            ).toBe(false);
          }
          if (buildIndex === 1) {
            expect(cacheInfo[`${prefix}|Compilation/modules`]).toEqual({
              hits: 0,
              misses: 0,
              reused: 0,
              stores: 2,
            });
          }
          buildIndex++;
        });
      },
    }),
  ],
});
