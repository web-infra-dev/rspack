const path = require('node:path');

module.exports = {
  plugins: [
    {
      apply(compiler) {
        let calls = 0;
        compiler.hooks.compilation.tap('RejectLoaderHook', (compilation) => {
          compiler.webpack.NormalModule.getCompilationHooks(
            compilation,
          ).loader.tap('RejectLoaderHook', (context) => {
            if (context.resourcePath === path.resolve(__dirname, 'input.js')) {
              calls++;
              throw new Error('Rejected loader hook');
            }
          });
        });
        compiler.hooks.afterCompile.tap('RejectLoaderHook', () => {
          // Hooks only borrow the native context. A failed hook must report
          // the module error while leaving the runner able to finish normally.
          expect(calls).toBe(1);
        });
      },
    },
  ],
};
