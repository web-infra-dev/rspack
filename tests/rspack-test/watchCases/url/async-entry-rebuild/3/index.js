const jsUrl = new URL('./target.js', import.meta.url);
const cssUrl = new URL('./target.css', import.meta.url);

it('should keep URL async entries unique across rebuilds', () => {
  expect(jsUrl.href).toMatch(/\/assets\/url-[^/]+\.js$/);
  expect(cssUrl.href).toMatch(/\/assets\/url-[^/]+\.css$/);
  expect(globalThis.URL_ENTRY_REBUILD_TARGET_EXECUTED).toBeUndefined();
});
