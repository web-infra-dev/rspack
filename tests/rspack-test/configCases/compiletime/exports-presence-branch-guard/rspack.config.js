/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  module: {
    parser: {
      javascript: {
        exportsPresence: 'warn',
      },
    },
  },
};
