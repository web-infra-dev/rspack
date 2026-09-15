import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default () => {
  return {
    extends: [import.meta.resolve('./base.rspack.config.mjs')],
    entry: './src/index.js',
    output: {
      path: path.resolve(__dirname, 'dist'),
    },
  };
};
