import fs from 'node:fs';
import path from 'node:path';

// `v` differs between steps so editing this file produces a real `change`.
export const v = 0;

it('exposes the recorded watch events file on the initial build', () => {
  const { changed, watcherChanged, aggregatedChanges } = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'recorded-events.json'), 'utf-8'),
  );
  expect(Array.isArray(changed)).toBe(true);
  expect(Array.isArray(watcherChanged)).toBe(true);
  expect(Array.isArray(aggregatedChanges)).toBe(true);
});
