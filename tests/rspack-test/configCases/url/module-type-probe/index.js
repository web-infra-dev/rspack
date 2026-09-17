const script = new URL('./replace-target.txt', import.meta.url);
const asset = new URL('./replace-target.js', import.meta.url);

it('should use the registered factory to determine URL target types without building the probe', () => {
  expect(script.pathname).toMatch(/^\/assets\/url-.*\.js$/);
  expect(asset.pathname).toBe('/assets/target.txt');
  expect(globalThis.URL_TYPE_PROBE_EXECUTED).toBeUndefined();
});
