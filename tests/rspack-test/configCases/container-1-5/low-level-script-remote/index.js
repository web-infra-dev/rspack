it('should load a script remote configured only through ContainerReferencePlugin', async () => {
  globalThis.lowLevelScriptRemote = {
    init() {},
    get() { return Promise.resolve(() => 42); },
  };
  try {
    expect((await import('standalone/value')).default).toBe(42);
  } finally {
    delete globalThis.lowLevelScriptRemote;
  }
});
