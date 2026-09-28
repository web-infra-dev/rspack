import zero from 'http://updates.example/0.js';
import one from 'http://updates.example/1.js';
import two from 'http://updates.example/2.js';
import three from 'http://updates.example/3.js';
import four from 'http://updates.example/4.js';
import five from 'http://updates.example/5.js';

it('keeps every concurrently fetched module in the lockfile', () => {
  expect([zero, one, two, three, four, five]).toEqual([0, 1, 2, 3, 4, 5]);
});
