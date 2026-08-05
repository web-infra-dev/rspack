/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  output: {
    module: false,
  },
  module: {
    parser: {
      javascript: {
        importMeta: {
          environment: false,
        },
      },
    },
  },
};
