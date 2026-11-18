const path = require('node:path');

const first = path.join(__dirname, 'first');
const second = path.join(__dirname, 'second');

module.exports = {
  context: __dirname,
  resolve: {
    alias: {
      '@': [first, second],
      '@params': [first + '?stale#stale', second + '?selected#fragment'],
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.contextModuleFactory.tap(
          'NonrecursiveAlias',
          (factory) => {
            factory.hooks.beforeResolve.tap('NonrecursiveAlias', (data) => {
              if (data !== false && data.request.includes('?nonrecursive')) {
                data.recursive = false;
              }
            });
          },
        );
      },
    },
  ],
  module: {
    rules: [
      {
        include: second,
        use: path.join(__dirname, 'loader.cjs'),
      },
    ],
  },
};
