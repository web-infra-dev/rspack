const path = require('node:path');
const { workerFunction } = require('@rspack/core');
const nested = workerFunction(path.join(__dirname, 'value.mjs'), { value: 20 });
const options = {
  nested,
  same: nested,
  map: new Map([['fn', nested]]),
  set: new Set([nested]),
};
options.self = options;
const fn = workerFunction('function-alias', options);
options.outer = fn;
module.exports = [false, true].map((parallel) => ({
  context: __dirname,
  loader: { prepared: workerFunction('./value.mjs', { value: 42 }) },
  resolveLoader: {
    alias: { 'function-alias': path.join(__dirname, 'outer.cjs') },
  },
  module: {
    rules: [
      {
        test: /input\.js$/,
        use: [
          {
            loader: path.join(__dirname, 'loader.cjs'),
            ident: 'worker-options',
            parallel,
            options: {
              fn,
              local: workerFunction('./value.mjs', { value: 22 }),
              parallel,
            },
          },
        ],
      },
    ],
  },
}));
