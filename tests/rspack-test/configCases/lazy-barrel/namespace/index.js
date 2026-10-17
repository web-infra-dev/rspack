import * as star from './lib/index.js';
import * as named from './lib/named.js';

it('preserves lazy reexports when the barrel namespace is consumed', () => {
  expect(Object.keys(star).sort()).toEqual(['create', 'createStore']);
  expect(Object.keys(named).sort()).toEqual(['create', 'createStore']);
  expect(star.createStore('star').state).toBe('star');
  expect(named.createStore('named').state).toBe('named');
  expect(star.create('local').state).toBe('local');
});
