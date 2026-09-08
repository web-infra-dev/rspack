import first from './first';
import second from './second';
import value from 'package';
import button from 'package/button';
import feature from 'package/feature/button';

it('matches layers and longest prefixes while retaining every provider', async () => {
  expect([value, button, feature, first, second]).toEqual([42, 42, 42, 42, 42]);
  const scopes = __webpack_require__.initializeSharingData.scopeToSharingDataMapping;
  const defaults = scopes.default;
  expect(defaults.find((item) => item.name === 'collision-a')).toBeDefined();
  expect(defaults.find((item) => item.name === 'collision-b')).toBeDefined();
  expect(defaults.find((item) => item.name === 'long/button')).toBeDefined();
  expect(defaults.some((item) => item.name.endsWith('feature/button'))).toBe(false);
  if (__webpack_layer__ === 'server') {
    expect(defaults.find((item) => item.name === 'short/button')).toBeDefined();
    expect(scopes.first.filter((item) => item.name === 'exact').map((item) => item.version).sort()).toEqual(['1.0.0', '2.0.0']);
    expect(scopes.second.find((item) => item.name === 'other').version).toBe('3.0.0');
  } else {
    expect(defaults.find((item) => item.name === 'fallback')).toBeDefined();
    expect(defaults.find((item) => item.name === 'fallback-short/button')).toBeDefined();
    expect(scopes.first).toBeUndefined();
    expect(scopes.second).toBeUndefined();
  }
  for (const items of Object.values(scopes)) {
    for (const item of items) {
      const factory = await item.factory();
      expect(factory()).toBe(42);
    }
  }
});
