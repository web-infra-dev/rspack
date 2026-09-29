import defer * as dep from './dep.js';
import { events } from './state.js';

export const read = () => dep.value;
export { dep };

it('should preserve deferred evaluation in a modern-module library', async () => {
  const lib = await import(/* webpackIgnore: true */ './bundle0.mjs');
  const namespace = lib.dep;
  expect(events).toEqual([]);

  expect(lib.read()).toBe(42);
  expect(lib.read()).toBe(42);
  expect(namespace.value).toBe(42);
  expect(events).toEqual(['dep']);
});
