const fs = require('fs/promises');
const path = require('path');

// Reusing a file version must restore its timestamp as well as its content.
// Keep both timestamps before the builds so safe-time checks allow cache hits.
const firstTime = new Date('2000-01-01T00:00:00Z');
const secondTime = new Date('2000-01-02T00:00:00Z');
const fileTimes = [firstTime, secondTime, firstTime, secondTime, secondTime];
let index = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  cache: {
    type: 'persistent',
  },
  module: {
    rules: [
      {
        test: /file\.js$/,
        use: {
          loader: './loader.js',
          options: { count: 0 },
        },
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.beforeRun.tapPromise('PLUGIN', async () => {
          const time = fileTimes[index];
          await fs.utimes(
            path.resolve(compiler.options.context, 'file.js'),
            time,
            time,
          );
        });

        let shouldRebuildFile = true;
        if (index == 0) {
          compiler.options.cache.readonly = false;
          shouldRebuildFile = true;
        } else if (index == 1) {
          compiler.options.cache.readonly = true;
          shouldRebuildFile = true;
        } else if (index == 2) {
          compiler.options.cache.readonly = true;
          shouldRebuildFile = false;
        } else if (index == 3) {
          compiler.options.cache.readonly = false;
          shouldRebuildFile = true;
        } else if (index == 4) {
          compiler.options.cache.readonly = true;
          shouldRebuildFile = false;
        }

        const loaderOptions = compiler.options.module.rules[0].use.options;
        compiler.hooks.done.tap('PLUGIN', function () {
          if (shouldRebuildFile) {
            expect(loaderOptions.count).toBe(1);
          } else {
            expect(loaderOptions.count).toBe(0);
          }
          // reset
          loaderOptions.count = 0;
          index++;
        });
      },
    },
  ],
};
