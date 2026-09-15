module.exports = {
  module: { rules: [{ test: /input\.js$/, use: './loader.js' }] },
  plugins: [
    {
      apply(compiler) {
        let retained;
        compiler.hooks.beforeRun.tap('RejectLoaderContext', () => {
          const plugin = compiler.__internal__builtinPlugins.find(
            (plugin) => plugin.name === 'JsLoaderRspackPlugin',
          );
          plugin.options = async (context) => {
            retained = context;
            throw new Error('Rejected loader invocation');
          };
        });
        compiler.hooks.afterCompile.tap('RejectLoaderContext', () => {
          expect(retained).toBeDefined();
          expect(() => retained.resource).toThrow('no longer available');
        });
      },
    },
  ],
};
