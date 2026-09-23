import defer * as dep from './dep.js';
import { events } from './state.js';

const before = events.slice();

it('should evaluate async dependencies without evaluating the deferred module', () => {
  expect(before).toEqual(['async']);
  expect(events).toEqual(['async']);
  expect(dep.value).toBe(42);
  expect(dep.value).toBe(42);
  expect(events).toEqual(['async', 'sync', 'dep']);
});
