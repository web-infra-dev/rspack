const fs = require('node:fs');
const path = require('node:path');
let runs = 0;
module.exports = {
  context: __dirname,
  mode: 'development',
  incremental: false,
  experiments: { newCache: { module: false, loader: true } },
  cache: { type: 'persistent' },
  snapshot: { module: { hash: true } },
  optimization: { minimize: false },
  module: {
    rules: [{ test: /consumer\.js$/, loader: './dependency-loader.js' }],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.beforeRun.tap('DependencyTimestamp', () => {
          const time = 1000000000;
          fs.utimesSync(path.join(__dirname, 'data.txt'), time, time);
        });
      },
    },
  ],
};
