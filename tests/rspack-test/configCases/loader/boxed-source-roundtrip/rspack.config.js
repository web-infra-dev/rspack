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
        const cachedKeys = [
          'content',
          'sourceMap',
          'additionalData',
          'resource',
          '_module',
          '__internal__loaderCache',
        ];
        const identities = new Map();
        const seen = new WeakSet();
        compiler.hooks.beforeRun.tap('BoxedSourceRoundtrip', () => {
          const plugin = compiler.__internal__builtinPlugins.find(
            (plugin) => plugin.name === 'JsLoaderRspackPlugin',
          );
          const run = plugin.options;
          plugin.options = async (context) => {
            const prototype = Object.getPrototypeOf(context);
            expect(prototype.constructor.name).toBe('JsLoaderContext');
            const reused = seen.has(context);
            seen.add(context);
            const resource = Object.getOwnPropertyDescriptor(
              prototype,
              'resource',
            ).get.call(context);
            if (identities.has(resource))
              expect(context).toBe(identities.get(resource));
            else identities.set(resource, context);
            let reads = 0;
            let commits = 0;
            const getterReads = {};
            for (const key of cachedKeys) {
              const descriptor = Object.getOwnPropertyDescriptor(
                prototype,
                key,
              );
              Object.defineProperty(context, key, {
                configurable: true,
                get() {
                  getterReads[key] = (getterReads[key] ?? 0) + 1;
                  if (key === 'content' || key === 'sourceMap') reads++;
                  return descriptor.get.call(context);
                },
              });
            }
            let stateReads = 0;
            let metadataReads = 0;
            let snapshot;
            const state = Object.getOwnPropertyDescriptor(prototype, 'state');
            const metadata = Object.getOwnPropertyDescriptor(
              prototype,
              'loaderItems',
            );
            Object.defineProperty(context, 'loaderItems', {
              configurable: true,
              get() {
                metadataReads++;
                const items = metadata.get.call(context);
                expect(items.every((item) => !('data' in item))).toBe(true);
                return items;
              },
            });
            Object.defineProperty(context, 'state', {
              configurable: true,
              get() {
                stateReads++;
                snapshot = state.get.call(context);
                expect(Object.getPrototypeOf(snapshot)).toBe(Object.prototype);
                expect(
                  snapshot.loaderItemStates.every(
                    (item) => !('loader' in item),
                  ),
                ).toBe(true);
                expect('loaderContextState' in snapshot).toBe(false);
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
            for (const count of Object.values(getterReads)) {
              expect(count).toBe(1);
            }
            // Restore native accessors before reusing the same class or checking
            // revoked access with snapshots from earlier entries.
            for (const key of [...cachedKeys, 'state', 'loaderItems'])
              delete context[key];
            contexts.push({ context, snapshot });
            if (snapshot.loaderState === 'Pitching') expect(reads).toBe(0);
          };
        });
        compiler.hooks.afterCompile.tap(
          'BoxedSourceRoundtrip',
          (compilation) => {
            expect(contexts.length).toBeGreaterThan(0);
            expect(contexts.length).toBeGreaterThan(identities.size);
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
