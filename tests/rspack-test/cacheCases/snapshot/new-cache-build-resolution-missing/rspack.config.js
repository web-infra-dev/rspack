const fs = require('node:fs');
const path = require('node:path');
module.exports = {
  context: __dirname,
  incremental: false,
  experiments: { newCache: { module: true, loader: false } },
  cache: { type: 'persistent', buildDependencies: ['build-dependency.js'] },
  snapshot: { module: { hash: true }, buildDependencies: { hash: true } },
  module: { rules: [{ test: /consumer\.js$/, loader: './count-loader.js' }] },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.beforeRun.tap('AddResolveCandidate', () => {
          if (
            fs.readFileSync(path.join(__dirname, 'step.txt'), 'utf8').trim() ===
            '1'
          ) {
            fs.writeFileSync(
              path.join(__dirname, 'preferred.js'),
              'module.exports = 1;',
            );
          }
        });
      },
    },
  ],
};
