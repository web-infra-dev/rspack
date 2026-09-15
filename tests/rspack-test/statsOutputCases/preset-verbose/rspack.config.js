const {
  LogTestPlugin,
} = require('@rspack/test-tools/helper/legacy/LogTestPlugin');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  incremental: false,
  mode: 'production',
  entry: './index',
  profile: true,
  stats: 'verbose',
  infrastructureLogging: {
    level: 'verbose',
  },
  plugins: [new LogTestPlugin()],
};
