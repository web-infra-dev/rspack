import { fileURLToPath } from 'node:url';

/**
 * @type {import('@rspack/core').Configuration}
 */
const config = {
  module: {
    rules: [
      {
        test: /index\.js$/,
        loader: fileURLToPath(import.meta.resolve('./loader.js')),
      },
    ],
  },
};
export default config;
