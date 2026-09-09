const fs = require('node:fs');
const path = require('node:path');
let step = 0;
let contextBuilds = 0;
module.exports = {
  context: __dirname,
  incremental: false,
  experiments: { newCache: { module: true, loader: false } },
  cache: { type: 'persistent' },
  snapshot: { module: { hash: true }, contextModule: { hash: true } },
  plugins: [
    {
      apply(compiler) {
        const directory = path.join(__dirname, 'context-fixture');
        compiler.hooks.beforeRun.tap('ChangeContext', () => {
          step = Number(
            fs.readFileSync(path.join(__dirname, 'step.txt'), 'utf8').trim(),
          );
          fs.writeFileSync(
            path.join(directory, 'a.js'),
            `module.exports = ${step < 2 ? 1 : 2};`,
          );
          fs.utimesSync(path.join(directory, 'a.js'), 1000000000, 1000000000);
          if (step === 3)
            fs.writeFileSync(
              path.join(directory, 'b.js'),
              'module.exports = 3;',
            );
          fs.utimesSync(directory, 1000000000, 1000000000);
        });
        compiler.hooks.compilation.tap('CountContextBuilds', (compilation) => {
          compilation.hooks.buildModule.tap('CountContextBuilds', (module) => {
            if (module.identifier().startsWith(`${directory}|`))
              contextBuilds++;
          });
          compilation.hooks.afterSeal.tap('CheckContextBuilds', () => {
            expect(contextBuilds).toBe(step < 2 ? 2 : step * 2);
          });
        });
      },
    },
  ],
};
