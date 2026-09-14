import a from './a';
import b from './b';

it('should add and build shared parser-created modules and their dependencies once', () => {
  for (const urls of [a, b]) {
    expect(urls).toHaveLength(3);
    expect(urls[0].pathname).toMatch(/^\/assets\/url-.*\.js$/);
    expect(urls[1].pathname).toMatch(/^\/assets\/url-.*\.js$/);
    expect(urls[2].pathname).toBe('/assets/target.txt');
  }
  expect(globalThis.URL_TYPE_PROBE_EXECUTED).toBeUndefined();
});
