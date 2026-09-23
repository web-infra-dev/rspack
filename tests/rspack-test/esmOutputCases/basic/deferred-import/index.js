import defer * as dep from './dep.js';
import defer * as ns from './namespace.js';
import defer * as deferredMixed from './mixed.js';
import { value as mixed } from './mixed.js';
import { events } from './state.js';

export const read = () => dep.value;
export const getNamespace = () => ns;
export const readNested = () => dep.readNested();
export { ns };

const before = events.slice();

it('should preserve deferred evaluation in a modern-module library', async () => {
  const lib = await import(/* webpackIgnore: true */ './main.mjs');
  expect(before).toEqual(['mixed']);
  expect(events).toEqual(['mixed']);
  expect(mixed).toBe(10);
  expect(deferredMixed.value).toBe(10);

  const namespace = lib.getNamespace();
  expect(lib.ns).toBe(namespace);
  expect(events).toEqual(['mixed']);

  expect(lib.read()).toBe(42);
  expect(lib.read()).toBe(42);
  expect(events).toEqual(['mixed', 'shared', 'dep']);

  expect(namespace.value).toBe(43);
  expect(lib.getNamespace()).toBe(namespace);
  expect(events).toEqual(['mixed', 'shared', 'dep', 'namespace']);

  expect(lib.readNested()).toBe(44);
  expect(lib.readNested()).toBe(44);
  expect(events).toEqual(['mixed', 'shared', 'dep', 'namespace', 'nested']);
});
