const jsUrl = new URL('./target.js', import.meta.url);

it('should remove the CSS URL entry when its dependency is removed', () => {
  expect(jsUrl.href).toMatch(/\/assets\/url-[^/]+\.js$/);
  expect(globalThis.URL_ENTRY_REBUILD_TARGET_EXECUTED).toBeUndefined();
});
