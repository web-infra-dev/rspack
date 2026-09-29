import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /value\.txt$/,
        use: { loader: './loader.mjs', options: { marker: true } },
      },
    ],
  },
  plugins: [
    definePlugin({
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
    }),
  ],
});
