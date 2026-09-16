/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  module: {
    parser: {
      javascript: {
        exportsPresence: 'auto',
      },
    },
  },
};
