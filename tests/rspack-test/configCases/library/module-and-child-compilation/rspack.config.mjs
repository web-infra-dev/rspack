import { fileURLToPath } from 'node:url';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  target: 'web',
  output: {
    module: true,
    library: {
      type: 'module',
    },
  },
  module: {
    parser: {
      javascript: {
        exportsPresence: 'error',
      },
    },
    rules: [
      {
        test: /\.custom$/i,
        loader: fileURLToPath(import.meta.resolve('./loader.js')),
      },
    ],
  },
};
