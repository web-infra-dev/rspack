import { LogTestPlugin } from '@rspack/test-tools/helper/legacy/LogTestPlugin';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  stats: 'errors-only',
  infrastructureLogging: {
    level: 'error',
  },
  plugins: [new LogTestPlugin()],
};
