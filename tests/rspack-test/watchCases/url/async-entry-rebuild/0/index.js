const jsUrl = new URL('./target.js', import.meta.url);

it('should keep URL async entries unique across rebuilds', () => {
  expect(jsUrl.href).toMatch(/\/assets\/url-[^/]+\.js$/);
  expect(globalThis.URL_ENTRY_REBUILD_TARGET_EXECUTED).toBeUndefined();
});
