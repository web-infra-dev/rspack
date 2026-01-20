const path = require('path');
const fs = require('fs/promises');

const libAIndex = path.resolve(__dirname, './lib/a/index');
const libTestIndex = path.resolve(__dirname, './lib/node_modules/test/index');
let index = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  experiments: {
    cache: {
      type: 'persistent',
      snapshot: {
        immutablePaths: [path.resolve(__dirname, './file.js')],
      },
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tapPromise('TestPlugin', async function () {
          index++;
          if (index === 1) {
            await fs.writeFile(libAIndex, String(index));
          }
          if (index === 2) {
            await fs.writeFile(libTestIndex, String(index));
          }
        });
      },
    },
  ],
};
