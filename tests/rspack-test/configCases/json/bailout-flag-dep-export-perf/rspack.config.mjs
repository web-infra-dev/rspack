/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  module: {
    parser: {
      json: {
        exportsDepth: Number.MAX_SAFE_INTEGER,
      },
    },
  },
};
