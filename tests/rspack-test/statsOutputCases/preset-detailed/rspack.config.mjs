import { LogTestPlugin } from '@rspack/test-tools/helper/legacy/LogTestPlugin';

/** @type {import("@rspack/core").Configuration} */
export default {
  incremental: false,
  mode: 'production',
  entry: './index',
  stats: 'detailed',
  infrastructureLogging: {
    level: 'log',
  },
  plugins: [new LogTestPlugin()],
};
