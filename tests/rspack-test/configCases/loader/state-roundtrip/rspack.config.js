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
        let calls = 0;
        compiler.hooks.beforeRun.tap('LoaderStateRoundtrip', () => {
          const plugin = compiler.__internal__builtinPlugins.find(
            (plugin) => plugin.name === 'JsLoaderRspackPlugin',
          );
          const run = plugin.options;
          plugin.options = async (context) => {
            calls++;
            const { state, loaderData } = context;
            expect(loaderData).toHaveLength(context.loaderItems.length);
            expect(
              state.loaderItemStates.every((item) => !('data' in item)),
            ).toBe(true);
            expect(Object.getPrototypeOf(state)).toBe(Object.prototype);
            expect(state.loaderItemStates).toHaveLength(
              context.loaderItems.length,
            );
            for (const item of context.loaderItems) {
              expect(Object.keys(item).sort()).toEqual([
                'cache',
                'loader',
                'type',
              ]);
              Object.freeze(item);
            }
            if (context.loaderState === 'Normal' && state.loaderIndex === 0) {
              expect(state.cacheable).toBe(false);
            }
            const result = await run(context);
            expect(result.state).toBe(state);
            expect(result.loaderData).toBe(loaderData);
            expect('loaderItems' in state).toBe(false);
            expect('resource' in state).toBe(false);
            // The return conversion consumes data and state, never metadata.
            Object.defineProperty(context, 'loaderItems', {
              get() {
                throw new Error('metadata must not be read on return');
              },
            });
            return result;
          };
        });
        compiler.hooks.afterCompile.tap('LoaderStateRoundtrip', () => {
          expect(calls).toBeGreaterThan(0);
        });
      },
    },
  ],
};
