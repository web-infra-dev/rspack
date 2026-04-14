import { copyFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

/** @type {import('prebundle').Config} */
export default {
  dependencies: [
    {
      name: 'jiti',
      afterBundle(task) {
        const distDir = join(task.distPath, 'dist');
        mkdirSync(distDir, { recursive: true });
        copyFileSync(
          join(task.depPath, 'dist/jiti.cjs'),
          join(distDir, 'jiti.cjs'),
        );
        copyFileSync(
          join(task.depPath, 'dist/babel.cjs'),
          join(distDir, 'babel.cjs'),
        );

        writeFileSync(
          join(task.distPath, 'index.js'),
          `const { createRequire } = require("node:module");
const _createJiti = require("./dist/jiti.cjs");

function onError(err) {
  throw err;
}

const nativeImport = (id) => import(id);

let _transform;
function lazyTransform(...args) {
  if (!_transform) {
    _transform = require("./dist/babel.cjs");
  }
  return _transform(...args);
}

function createJiti(id, opts = {}) {
  if (!opts.transform) {
    opts = { ...opts, transform: lazyTransform };
  }
  return _createJiti(id, opts, {
    onError,
    nativeImport,
    createRequire,
  });
}

module.exports = createJiti;
module.exports.createJiti = createJiti;
`,
        );
      },
    },
  ],
};
