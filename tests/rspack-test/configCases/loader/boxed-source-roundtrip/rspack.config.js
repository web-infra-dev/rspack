const path = require('node:path');

module.exports = {
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /input\.js$/,
        use: ['./verify.js', 'builtin:test-passthrough-loader', './produce.js'],
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        const contexts = [];
        compiler.hooks.beforeRun.tap('BoxedSourceRoundtrip', () => {
          const plugin = compiler.__internal__builtinPlugins.find(
            (plugin) => plugin.name === 'JsLoaderRspackPlugin',
          );
          const run = plugin.options;
          plugin.options = async (context) => {
            const prototype = Object.getPrototypeOf(context);
            expect(prototype.constructor.name).toBe('JsLoaderContext');
            let reads = 0;
            let commits = 0;
            for (const key of ['content', 'sourceMap']) {
              const descriptor = Object.getOwnPropertyDescriptor(
                prototype,
                key,
              );
              Object.defineProperty(context, key, {
                get() {
                  reads++;
                  return descriptor.get.call(context);
                },
              });
            }
            let stateReads = 0;
            let metadataReads = 0;
            let snapshot;
            let reused;
            const state = Object.getOwnPropertyDescriptor(prototype, 'state');
            const metadata = Object.getOwnPropertyDescriptor(
              prototype,
              'loaderItems',
            );
            Object.defineProperty(context, 'loaderItems', {
              get() {
                metadataReads++;
                const items = metadata.get.call(context);
                expect(items.every((item) => !('data' in item))).toBe(true);
                return items;
              },
            });
            Object.defineProperty(context, 'state', {
              get() {
                stateReads++;
                snapshot = state.get.call(context);
                expect(Object.getPrototypeOf(snapshot)).toBe(Object.prototype);
                expect(
                  snapshot.loaderItemStates.every(
                    (item) => !('loader' in item),
                  ),
                ).toBe(true);
                reused = !!snapshot.loaderContextState;
                if (
                  snapshot.loaderState === 'Normal' &&
                  snapshot.loaderIndex === 0
                ) {
                  expect(snapshot.cacheable).toBe(false);
                }
                return snapshot;
              },
              set(value) {
                commits++;
                expect(value).toBe(snapshot);
                state.set.call(context, value);
              },
            });
            expect(await run(context)).toBe(context);
            expect(stateReads).toBe(1);
            expect(commits).toBe(1);
            expect(metadataReads).toBe(reused ? 0 : 1);
            contexts.push({ context, snapshot });
            if (snapshot.loaderState === 'Pitching') expect(reads).toBe(0);
          };
        });
        compiler.hooks.afterCompile.tap(
          'BoxedSourceRoundtrip',
          (compilation) => {
            expect(contexts.length).toBeGreaterThan(0);
            for (const { context, snapshot } of contexts) {
              expect(() => context.content).toThrow('no longer available');
              expect(() => context._module).toThrow('no longer available');
              expect(() => context.state).toThrow('no longer available');
              // Owned state remains readable after the native class is revoked.
              expect(Array.isArray(snapshot.loaderItemStates)).toBe(true);
              expect(() => {
                context.state = snapshot;
              }).toThrow('no longer available');
              expect(() => {
                context.__internal__error = new Error('late write');
              }).toThrow('no longer available');
            }
            const modules = [...compilation.modules].filter((module) =>
              module.resource?.includes('input.js'),
            );
            expect(modules).toHaveLength(2);
            expect(
              compilation.fileDependencies.has(
                path.join(__dirname, 'produce.js'),
              ),
            ).toBe(true);
          },
        );
      },
    },
  ],
};
