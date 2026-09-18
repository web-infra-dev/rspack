import path from 'node:path';
import fs from 'node:fs/promises';

let content = 1;

const logA = path.join(import.meta.dirname, './configs/a.log');
const logB = path.join(import.meta.dirname, './configs/b.log');
const BuildDependency = path.join(import.meta.dirname, './configs/index.js');

/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    buildDependencies: [BuildDependency],
    snapshot: {
      immutablePaths: [path.join(import.meta.dirname, 'immutable')],
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.beforeCompile.tapPromise(
          'Test Plugin',
          async function () {
            if (content == 1) {
              // init
              await fs.writeFile(logA, String(content));
              await fs.writeFile(logB, String(content));
            } else if (content == 2) {
              // do nothing
            } else if (content == 3) {
              // update a.log
              await fs.writeFile(logA, String(content));
            } else if (content == 4) {
              // update b.log
              await fs.writeFile(logB, String(content));
            } else if (content == 5) {
              // do nothing
            } else if (content == 6) {
              await fs.writeFile(BuildDependency, String(content));
            }
            content++;
          },
        );
      },
    },
  ],
};
