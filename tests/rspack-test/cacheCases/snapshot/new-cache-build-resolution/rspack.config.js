const fs = require('node:fs');
const path = require('node:path');
module.exports = {
  context: __dirname,
  incremental: false,
  experiments: { newCache: { module: true, loader: false } },
  cache: { type: 'persistent', buildDependencies: ['build-dependency.js'] },
  snapshot: {
    module: { hash: true },
    buildDependencies: { hash: true },
    resolveBuildDependencies: { hash: true },
  },
  module: { rules: [{ test: /consumer\.js$/, loader: './count-loader.js' }] },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.beforeRun.tap('PreserveTimestamp', () => {
          fs.utimesSync(
            path.join(__dirname, 'resolution-package/package.json'),
            1000000000,
            1000000000,
          );
        });
      },
    },
  ],
};
