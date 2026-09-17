export default {
  module: {
    rules: [
      {
        test: /value\.txt$/,
        use: { loader: './loader.js', options: { marker: true } },
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('LoaderHookContext', (compilation) => {
          compiler.webpack.NormalModule.getCompilationHooks(
            compilation,
          ).loader.tap('LoaderHookContext', (context) => {
            if (!context.resourcePath.endsWith('value.txt')) return;
            expect(context.loaderIndex).toBe(0);
            expect(context.getOptions()).toEqual({ marker: true });
            context.cacheable(false);
            context.data = { hookData: true };
          });
        });
      },
    },
  ],
};
