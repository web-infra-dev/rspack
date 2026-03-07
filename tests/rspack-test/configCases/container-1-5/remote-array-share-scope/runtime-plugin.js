const path = require('path');

// default mode will use fallback asset if no server data. And the fallback will load asset via fetch + eval.
// Cause the asset not deploy, so we need to proxy the asset to local.
module.exports = function () {
  return {
    name: 'proxy-shared-asset',
    loadEntry: ({ remoteInfo }) => {
      const { entryGlobalName } = remoteInfo;
      globalThis[entryGlobalName] =
          __non_webpack_require__(path.resolve(__dirname, 'remoteEntry.js'))[entryGlobalName];
        return globalThis[entryGlobalName];
    },
  };
};
