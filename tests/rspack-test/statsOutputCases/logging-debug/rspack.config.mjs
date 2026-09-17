import { LogTestPlugin } from '@rspack/test-tools/helper/legacy/LogTestPlugin';
import { fileURLToPath } from 'node:url';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  performance: false,
  module: {
    rules: [
      {
        test: /index\.js$/,
        use: fileURLToPath(
          import.meta.resolve('../logging/node_modules/custom-loader/index.js'),
        ),
      },
    ],
  },
  plugins: [new LogTestPlugin(true)],
  stats: {
    assets: true,
    modules: true,
    colors: true,
    logging: false,
    loggingDebug: /custom-loader/,
  },
};
