import { load as loadA } from './a';
import { load as loadB } from './b';

it('should keep fallback resources associated with their configured imports', async () => {
  // Discover providers independently of the aliased consume requests.
  const loadProviders = () => Promise.all([
    import('pkg-a'),
    import('pkg-b'),
    import('alias-b'),
    import('pkg-a-copy'),
    import('pkg-a?copy#fragment'),
    import('alias-a-query'),
    import('pkg/sub'),
    import('mapped/a'),
  ]);
  expect(typeof loadProviders).toBe('function');
  expect(typeof loadA).toBe('function');
  expect(typeof loadB).toBe('function');

  const fallbacks = __webpack_require__.federation.sharedFallback;
  for (const [key, expected] of [
    ['same-key', [
      ['1.0.0', 'a'],
      ['1.0.0', 'a'],
      ['1.0.0', 'query'],
      ['2.0.0', 'b'],
      ['3.0.0', 'b'],
    ]],
    ['prefix/sub', [['1.0.0', 'sub']]],
    ['mapped/a', [['1.0.0', 'a']]],
  ]) {
    const actual = fallbacks[key].map(([entry, version, globalName]) => {
      const container = eval('require')(`./${entry}`)[globalName];
      return [version, container.get()().value];
    });
    expect(actual.sort()).toEqual(expected.sort());
  }
  for (const [key, expected] of [
    ['relative', ['relative-a', 'relative-b']],
    ['same-key', ['b', 'query']],
  ]) {
    const handlers = Object.values(
      __webpack_require__.federation.consumesLoadingModuleToHandlerMapping,
    ).filter(({ shareKey }) => shareKey === key);
    for (const [entry, , globalName] of fallbacks[key]) {
      globalThis[globalName] = eval('require')(`./${entry}`)[globalName];
    }
    const values = await Promise.all(handlers.map(async ({ getter }) => {
      const factory = await getter();
      return factory().value;
    }));
    expect(values.sort()).toEqual(expected);
    for (const [, , globalName] of fallbacks[key]) {
      delete globalThis[globalName];
    }
  }
});
