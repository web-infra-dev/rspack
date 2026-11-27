module.exports = () => ({
  name: 'load-local-full-fallback',
  loadEntry({ remoteInfo }) {
    const { entry, entryGlobalName } = remoteInfo;
    globalThis[entryGlobalName] = eval('require')(entry)[entryGlobalName];
    return globalThis[entryGlobalName];
  }
});
