import path from 'node:path';
import fs from 'node:fs/promises';

let index = -1;
let content = 0;

const buildDependency = path.join(import.meta.dirname, 'other.config.js');
const libFileA = path.join(import.meta.dirname, 'node_modules/lib/src/a.js');
const libFileB = path.join(import.meta.dirname, 'node_modules/lib/src/b.js');
const libFileIndex = path.join(
  import.meta.dirname,
  'node_modules/lib/src/index.js',
);
const libPackageJson = path.join(
  import.meta.dirname,
  'node_modules/lib/package.json',
);
const projectDep = path.join(import.meta.dirname, 'dep.js');
const projectPackageJson = path.join(import.meta.dirname, 'package.json');

/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    buildDependencies: [buildDependency],
    snapshot: {
      immutablePaths: [path.join(import.meta.dirname, './file.js')],
      unmanagedPaths: [path.join(import.meta.dirname, 'node_modules/lib')],
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.beforeCompile.tapPromise(
          'Test Plugin',
          async function () {
            // NEXT_START closes the previous compiler and flushes its cache
            // before this hook runs. Keep the initial build unchanged.
            if (index === -1) {
              index++;
              return;
            }
            if (index === 0) {
              await fs.writeFile(projectDep, String(content));
            } else if (index === 1) {
              await fs.writeFile(libFileA, String(content));
            } else if (index === 2) {
              await fs.writeFile(libFileB, String(content));
            } else if (index === 3) {
              await fs.writeFile(libFileIndex, String(content));
            } else if (index === 4) {
              const content = await fs.readFile(libPackageJson);
              await fs.writeFile(
                libPackageJson,
                content.toString().replace('0.0.1', '0.0.2'),
              );
            } else if (index === 5) {
              const content = await fs.readFile(projectPackageJson);
              await fs.writeFile(
                projectPackageJson,
                content.toString().replace('0.0.1', '0.0.2'),
              );
            }
            index++;
            content++;
          },
        );
      },
    },
  ],
};
