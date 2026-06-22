/** @type {import("@rspack/core").Configuration} */
module.exports = {
  cache: {
    type: 'persistent',
  },
  snapshot: {
    dependencies: {
      hash: true,
    },
    contextDependencies: {
      timestamp: true,
    },
  },
};
