import { LogTestPlugin } from '@rspack/test-tools/helper/legacy/LogTestPlugin';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  stats: false,
  plugins: [new LogTestPlugin()],
};
