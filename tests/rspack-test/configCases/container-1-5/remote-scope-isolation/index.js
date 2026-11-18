require('./primary');
require('./secondary');
require('./isolated');

it('should initialize only matching remotes and preserve each remote scope contract', async () => {
  try {
    await __webpack_init_sharing__(['primary', 'secondary']);
    expect(globalThis.__mfScopeInits).toEqual([{
      scope: __webpack_share_scopes__.secondary,
      keys: ['secondary', 'primary'],
    }]);

    expect((await import('isolated/value')).default).toBe('remote');
    expect(globalThis.__mfScopeInits).toHaveLength(2);
    const isolated = globalThis.__mfScopeInits[1];
    expect(isolated.scope).toBe(__webpack_share_scopes__.isolated);
    expect(isolated.keys).toBe('isolated');
    expect((await isolated.scope.value['1.0.0'].get())()).toBe('isolated');

    expect((await import('ordered/value')).default).toBe('remote');
    expect(globalThis.__mfScopeInits).toHaveLength(2);
    expect((await __webpack_share_scopes__.secondary.value['1.0.0'].get())()).toBe('secondary');
    expect(__webpack_share_scopes__.primary).not.toBe(__webpack_share_scopes__.isolated);
  } finally {
    delete globalThis.__mfScopeInits;
  }
});
