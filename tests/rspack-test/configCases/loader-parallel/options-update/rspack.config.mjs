import { NormalModule } from '@rspack/core';
export default {
  cache: false,
  incremental: false,
  module: {
    rules: [
      { test: /values\.js$/, loader: './entry-loader.js' },
      {
        test: /value\.js$/,
        use: [
          {
            loader: './loader.js',
            parallel: true,
            options: { revision: -1, shared: new SharedArrayBuffer(4) },
          },
        ],
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('UpdateOptions', (compilation) => {
          NormalModule.getCompilationHooks(compilation).loader.tap(
            'UpdateOptions',
            (context) => {
              if (context.resourcePath.endsWith('/value.js')) {
                const options = {
                  revision: Number(context.resourceQuery.slice(1)),
                };
                context.loaders[context.loaderIndex].options = options;
                // Exercise both content invalidation and non-V8-serializable values.
                if (options.revision % 2 === 0)
                  options.shared = new SharedArrayBuffer(4);
              }
            },
          );
        });
      },
    },
  ],
};
