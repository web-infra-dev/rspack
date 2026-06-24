const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  cache: {
    type: 'persistent',
    maxVersions: 1,
    storage: {
      type: 'filesystem',
    },
    snapshot: {
      immutablePaths: [path.resolve(__dirname, './file.js')],
    },
    portable: true,
  },
};
