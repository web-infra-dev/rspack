const path = require('path');
const fs = require('fs/promises');

const libAIndex = path.resolve(__dirname, './lib/a/index');
// Both timestamps must precede the builds so safe-time checks allow cache hits.
const initialTime = new Date('2000-01-01T00:00:00Z');
const modifiedTime = new Date('2000-01-02T00:00:00Z');
let index = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  cache: {
    type: 'persistent',
    snapshot: {
      immutablePaths: [path.resolve(__dirname, './file.js')],
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.beforeRun.tapPromise('TestPlugin', async function () {
          if (index === 0) {
            await fs.utimes(libAIndex, initialTime, initialTime);
          }
        });
        compiler.hooks.done.tapPromise('TestPlugin', async function () {
          index++;
          if (index === 1) {
            // Change only a context member's timestamp, keeping its contents stable.
            await fs.utimes(libAIndex, modifiedTime, modifiedTime);
          }
        });
      },
    },
  ],
};
