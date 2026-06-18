/** @type {import("@rspack/core").Configuration} */
module.exports = {
  cache: {
    type: 'persistent',
  },
  snapshot: {
    contextModule: {
      timestamp: true,
    },
  },
};
