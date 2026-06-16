import fs from 'node:fs';
import path from 'node:path';

export const v = 1;

const endsWith = suffix => file => file.replace(/\\/g, '/').endsWith(suffix);

it('emits a `change` event for the edited file via WatchFileSystem.on', () => {
  const { changed } = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'recorded-events.json'), 'utf-8'),
  );
  expect(changed.some(endsWith('/index.js'))).toBe(true);
});

it('emits the same `change` through the watchpack-compatible watcher shim', () => {
  const { watcherChanged } = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'recorded-events.json'), 'utf-8'),
  );
  expect(watcherChanged.some(endsWith('/index.js'))).toBe(true);
});

it('emits an `aggregated` event including the edited file', () => {
  const { aggregatedChanges } = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'recorded-events.json'), 'utf-8'),
  );
  expect(aggregatedChanges.some(endsWith('/index.js'))).toBe(true);
});
