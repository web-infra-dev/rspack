const path = require('node:path');
const {
  CssExtractRspackPlugin,
  NormalModule,
  workerFunction,
} = require('@rspack/core');

module.exports = [false, true].flatMap((parallel) =>
  [false, true].flatMap((runtime) =>
    ['worker', 'mixed', 'interceptor'].map((mode) => ({
      target: 'node',
      loader: {
        readHookWorker() {
          return this.hookWorker;
        },
      },
      module: {
        rules: [
          {
            test: /\.css$/,
            type: 'javascript/auto',
            use: [
              {
                loader: path.join(__dirname, 'probe.cjs'),
                parallel,
                options: {
                  runtime,
                  hookWorker: parallel && mode === 'worker',
                  mode,
                },
              },
              { loader: CssExtractRspackPlugin.loader, parallel },
              { loader: 'css-loader', parallel },
            ],
          },
        ],
      },
      plugins: [
        new CssExtractRspackPlugin({ runtime }),
        {
          apply(compiler) {
            compiler.hooks.thisCompilation.tap(
              'CheckLoaderHook',
              (compilation) => {
                const hook =
                  NormalModule.getCompilationHooks(compilation).loader;
                hook.tap(
                  'CheckWorker',
                  workerFunction(path.join(__dirname, 'hook.mjs'), { runtime }),
                );
                if (mode === 'mixed') {
                  hook.tap('CheckMain', (context) => {
                    expect(
                      context[Symbol.for('css-extract-rspack-plugin')],
                    ).toEqual({ runtime });
                    expect(context.hookWorker).toBe(false);
                    context.hookOrder.push('main');
                  });
                }
                if (mode === 'interceptor') {
                  hook.intercept({
                    call(context) {
                      context.hookOrder = ['interceptor'];
                    },
                  });
                }
                compilation.hooks.processAssets.tap('CheckCss', (assets) => {
                  expect(assets['main.css'].source().toString()).toContain(
                    'color: red',
                  );
                });
              },
            );
          },
        },
      ],
    })),
  ),
);
