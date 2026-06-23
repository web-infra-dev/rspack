const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  cache: {
    type: 'persistent',
    storage: {
      type: 'filesystem',
      maxVersions: 1,
    },
    snapshot: {
      immutablePaths: [path.resolve(__dirname, './file.js')],
    },
    portable: true,
  },
};
