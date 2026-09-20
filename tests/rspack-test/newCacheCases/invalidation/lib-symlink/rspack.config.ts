import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';
import fs from 'node:fs';
import fsPromise from 'node:fs/promises';

let index = 1;
const nodeModulesPath = path.join(import.meta.dirname, './node_modules');
const toolsV1 = path.join(import.meta.dirname, 'libs/tools_v1');
const toolsV2 = path.join(import.meta.dirname, 'libs/tools_v2');
const libLinkedPath = path.join(nodeModulesPath, 'tools');

export default defineConfig({
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    snapshot: {
      immutablePaths: [path.join(import.meta.dirname, 'immutable')],
      managedPaths: [path.join(import.meta.dirname, './libs'), nodeModulesPath],
    },
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        // make sure the node_modules dir exist
        fs.mkdirSync(nodeModulesPath, { recursive: true });
        compiler.hooks.beforeCompile.tapPromise(
          'Test Plugin',
          async function () {
            if (index === 1) {
              await fsPromise.symlink(toolsV1, libLinkedPath);
            } else {
              await fsPromise.unlink(libLinkedPath);
              await fsPromise.symlink(toolsV2, libLinkedPath);
            }
            index++;
          },
        );
      },
    }),
  ],
});
