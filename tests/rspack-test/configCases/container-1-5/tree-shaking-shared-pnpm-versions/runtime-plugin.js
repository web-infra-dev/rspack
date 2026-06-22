module.exports = function () {
  return {
    name: 'proxy-shared-asset',
    loadEntry: ({ remoteInfo }) => {
      const { entry, entryGlobalName } = remoteInfo;
      if (entry.includes('PUBLIC_PATH')) {
        const relativePath = entry.replace('PUBLIC_PATH', './');
        globalThis[entryGlobalName] =
          eval("require")(relativePath)[entryGlobalName];
        return globalThis[entryGlobalName];
      }
    },
  };
};
