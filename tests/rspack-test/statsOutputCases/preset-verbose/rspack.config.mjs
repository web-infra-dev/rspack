import { LogTestPlugin } from '@rspack/test-tools/helper/legacy/LogTestPlugin';

/** @type {import("@rspack/core").Configuration} */
export default {
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
