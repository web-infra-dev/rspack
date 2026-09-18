import path from 'node:path';

const file = path.resolve(import.meta.dirname, 'lib.js');
const createUse = (loaders) => loaders.map((l) => ({ loader: l }));
/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: file,
        resourceQuery: /case-1/,
        use: createUse(['./raw.mjs', './string.mjs', './raw.mjs']),
      },
      {
        test: file,
        resourceQuery: /case-2/,
        use: createUse(['./string.mjs', './raw.mjs', './string.mjs']),
      },
    ],
  },
};
