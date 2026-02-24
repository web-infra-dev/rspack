// const path = require('path');

// default mode will use fallback asset if no server data. And the fallback will load asset via fetch + eval.
// Cause the asset not deploy, so we need to proxy the asset to local.
module.exports = function () {
  return {
    name: 'proxy-shared-asset',
    loadEntry: ({ remoteInfo }) => {
      const { entry, entryGlobalName } = remoteInfo;
      const relativePath = entry.replace('http://localhost:3001/', './');
      globalThis[entryGlobalName] =
          __non_webpack_require__(relativePath)[entryGlobalName];
        return globalThis[entryGlobalName];
    },
  };
};
