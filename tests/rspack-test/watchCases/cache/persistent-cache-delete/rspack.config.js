const path = require('path');
const { DefinePlugin } = require('@rspack/core');

/** @type {(_env: unknown, args: { tempPath: string }) => import("@rspack/core").Configuration} */
module.exports = (_env, { tempPath }) => {
  const cacheDir = path.join(tempPath, 'node_modules/.cache/rspack');

  return {
    mode: 'development',
    cache: {
      type: 'persistent',
      storage: {
        type: 'filesystem',
        directory: cacheDir,
      },
    },
    watchOptions: {
      ignored: /node_modules/,
    },
    plugins: [
      new DefinePlugin({
        __CACHE_DIR__: JSON.stringify(cacheDir),
      }),
    ],
  };
};
