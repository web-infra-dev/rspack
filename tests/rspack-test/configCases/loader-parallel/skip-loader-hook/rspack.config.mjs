import { NormalModule } from '@rspack/core';

export default {
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: [
          './main-loader.js',
          {
            loader: './worker-loader.js',
            parallel: { maxWorkers: 2 },
            options: {},
          },
        ],
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'SkipParallelLoaderHook',
          (compilation) => {
            NormalModule.getCompilationHooks(compilation).loader.tap(
              'SkipParallelLoaderHook',
              (context) => {
                if (context.loaders[context.loaderIndex]?.parallel)
                  throw new Error(
                    'The worker pitch/normal task reached the loader hook',
                  );
                context.fromMainHook = 'main';
              },
            );
          },
        );
      },
    },
  ],
};
