/** @type {import("@rspack/core").Configuration} */
module.exports = {
  cache: {
    type: 'persistent',
  },
  snapshot: {
    module: {
      hash: true,
    },
    contextModule: {
      timestamp: true,
    },
  },
};
