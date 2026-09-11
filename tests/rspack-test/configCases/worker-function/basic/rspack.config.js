const path = require('node:path');
const { workerFunction } = require('@rspack/core');
module.exports = {
  context: __dirname,
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.normalModuleFactory.tap(
          'WorkerFunctionTest',
          (factory) => {
            const hook = factory.hooks.beforeResolve;
            hook.tap('first', (data) => {
              if (data.request === './original') data.request = './step-1';
            });
            hook.tapPromise('last', async (data) => {
              if (data.request === './step-3') data.request = './answer.js';
              if (data.request === './ignored')
                throw new Error('worker bail did not stop the main segment');
            });
            hook.tapPromise(
              { name: 'worker', before: 'last' },
              workerFunction(path.join(__dirname, 'resolve.cjs'), {
                from: './step-1',
                to: './step-2',
                mainThread: () => require('node:worker_threads').isMainThread,
                nested: workerFunction(path.join(__dirname, 'suffix.mjs'), {
                  suffix: '2',
                }),
              }),
            );
            hook.tapPromise(
              { name: 'second-worker', before: 'last' },
              workerFunction(path.join(__dirname, 'resolve.cjs'), {
                from: './step-2',
                to: './step-3',
              }),
            );
          },
        );
      },
    },
  ],
};
