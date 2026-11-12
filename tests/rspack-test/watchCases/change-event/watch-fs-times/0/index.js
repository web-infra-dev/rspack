import fs from 'node:fs';
import path from 'node:path';

// Registers a context dependency, so the context time table is non-empty.
require.context('./ctx').keys();

it('exposes the recorded watch times file on the initial build', () => {
  const probe = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'watch-times.json'), 'utf-8'),
  );
  // The initial build runs before the first `watch()` call, so the tables are
  // still empty here — only their shape is asserted.
  expect(typeof probe.times).toBe('object');
  expect(Array.isArray(probe.timeInfoEntries)).toBe(true);
  expect(Array.isArray(probe.collectedFiles)).toBe(true);
  expect(Array.isArray(probe.collectedContexts)).toBe(true);
});
