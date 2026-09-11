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
            contexts.push(context);
            const prototype = Object.getPrototypeOf(context);
            expect(prototype.constructor.name).toBe('JsLoaderContext');
            const pitch = context.loaderState === 'Pitching';
            if (!pitch && context.loaderIndex === 0) {
              expect(context.cacheable).toBe(false);
            }
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
            const result = Object.getOwnPropertyDescriptor(
              prototype,
              '__internal__result',
            );
            Object.defineProperty(context, '__internal__result', {
              set(value) {
                commits++;
                result.set.call(context, value);
              },
            });
            expect(await run(context)).toBe(context);
            expect(commits).toBe(1);
            if (pitch) expect(reads).toBe(0);
          };
        });
        compiler.hooks.afterCompile.tap(
          'BoxedSourceRoundtrip',
          (compilation) => {
            expect(contexts.length).toBeGreaterThan(0);
            for (const context of contexts) {
              expect(() => context.content).toThrow('no longer available');
              expect(() => context._module).toThrow('no longer available');
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
