import data from './data.json' with { type: 'json' };
import './plain';

export { default as reexported } from './reexport.json' with { type: 'json' };

export function load() {
  return import('./async.json', { with: { type: 'json' } });
}

export function loadContext(name) {
  return import(`./context/${name}.json`, { with: { type: 'json' } });
}

it('should import json with attributes', () => {
  expect(data.value).toBe('data');
});

it('should import context json with attributes', async () => {
  const mod = await loadContext('item');
  expect(mod.default.value).toBe('context');
});
