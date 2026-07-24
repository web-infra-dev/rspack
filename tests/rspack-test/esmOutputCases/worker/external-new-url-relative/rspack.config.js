/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: {
    main: './index.js',
    'worker-source': './worker.js',
  },
  externalsType: 'modern-module',
  externals: [
    ({ request, contextInfo }, callback) => {
      if (contextInfo.issuer && request === './worker.js') {
        callback(undefined, './worker-source.mjs');
        return;
      }
      callback();
    },
  ],
  module: {
    parser: {
      javascript: {
        worker: {
          url: 'new-url-relative',
        },
      },
    },
  },
};
