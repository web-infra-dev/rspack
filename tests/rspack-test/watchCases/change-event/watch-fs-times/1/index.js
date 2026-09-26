import fs from 'node:fs';
import path from 'node:path';

require.context('./ctx').keys();

const endsWith = (suffix) => (p) => p.replace(/\\/g, '/').endsWith(suffix);

const readProbe = () =>
  JSON.parse(fs.readFileSync(path.join(__dirname, 'watch-times.json'), 'utf-8'));

it('populates compiler file/context timestamps from the watch callback', () => {
  const { fileTimestamps, contextTimestamps } = readProbe();

  const file = fileTimestamps.find(([p]) => endsWith('/index.js')(p));
  expect(file).toBeTruthy();
  expect(typeof file[1].safeTime).toBe('number');
  expect(file[1].safeTime).toBeGreaterThan(0);
  expect(typeof file[1].timestamp).toBe('number');

  const context = contextTimestamps.find(([p]) => endsWith('/ctx')(p));
  expect(context).toBeTruthy();
  expect(typeof context[1].safeTime).toBe('number');
  expect(context[1].safeTime).toBeGreaterThan(0);

  // Like watchpack: a file found below the context, though never registered
  // as a file dependency, is in the table too.
  const scanned = fileTimestamps.find(([p]) => endsWith('/ctx/keep.js')(p));
  expect(scanned).toBeTruthy();
  expect(typeof scanned[1].safeTime).toBe('number');
});

it('reports the same paths through getTimes / getTimeInfoEntries', () => {
  const { times, timeInfoEntries } = readProbe();

  const indexPath = Object.keys(times).find(endsWith('/index.js'));
  expect(indexPath).toBeTruthy();
  expect(typeof times[indexPath]).toBe('number');
  expect(times[indexPath]).toBeGreaterThan(0);

  const file = timeInfoEntries.find(([p]) => endsWith('/index.js')(p));
  expect(file).toBeTruthy();
  expect(typeof file[1].safeTime).toBe('number');

  const context = timeInfoEntries.find(([p]) => endsWith('/ctx')(p));
  expect(context).toBeTruthy();
  expect(typeof context[1].safeTime).toBe('number');
});

it('fills the caller-provided maps through collectTimeInfoEntries', () => {
  const { collectedFiles, collectedContexts } = readProbe();

  expect(collectedFiles.some(([p]) => endsWith('/index.js')(p))).toBe(true);
  expect(collectedContexts.some(([p]) => endsWith('/ctx')(p))).toBe(true);
});

it('exposes the times API on the watchpack-compatible watcher as well', () => {
  const { watcherTimes } = readProbe();

  expect(Object.keys(watcherTimes).some(endsWith('/index.js'))).toBe(true);
});
