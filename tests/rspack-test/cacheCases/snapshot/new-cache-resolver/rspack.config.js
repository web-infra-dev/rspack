const fs = require('node:fs');
const path = require('node:path');
module.exports = {
  context: __dirname,
  incremental: false,
  experiments: { newCache: { module: false, loader: false } },
  cache: { type: 'persistent' },
  snapshot: { managedPaths: [], resolve: { hash: true } },
  module: { rules: [{ test: /consumer\.js$/, loader: './loader-package' }] },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.beforeRun.tap('ChangeResolution', () => {
          const step = Number(
            fs.readFileSync(path.join(__dirname, 'step.txt'), 'utf8').trim(),
          );
          if (step === 2)
            fs.writeFileSync(
              path.join(__dirname, 'pick.js'),
              'module.exports = 2;',
            );
          for (const metadata of [
            'node_modules/conditional/package.json',
            'loader-package/package.json',
          ]) {
            fs.utimesSync(
              path.join(__dirname, metadata),
              1000000000,
              1000000000,
            );
          }
        });
        compiler.hooks.afterCompile.tap(
          'CheckResolveDependencies',
          (compilation) => {
            // These must also be propagated by restored resolver cache entries.
            expect(
              compilation.fileDependencies.has(
                path.join(__dirname, 'node_modules/conditional/package.json'),
              ),
            ).toBe(true);
            if (!fs.existsSync(path.join(__dirname, 'pick.js'))) {
              expect(
                compilation.missingDependencies.has(
                  path.join(__dirname, 'pick.js'),
                ),
              ).toBe(true);
            }
          },
        );
      },
    },
  ],
};
