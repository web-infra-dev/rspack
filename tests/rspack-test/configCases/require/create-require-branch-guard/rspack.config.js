/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  module: {
    parser: {
      javascript: {
        createRequire: 'createRequire from ./shim.js',
      },
    },
  },
};
