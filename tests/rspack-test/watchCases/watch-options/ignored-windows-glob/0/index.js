import fs from 'node:fs';
import path from 'node:path';

it('matches Windows paths with a Windows-style ignored glob', () => {
  const { matchesWindowsPath } = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'ignored-glob.json'), 'utf-8'),
  );
  // NativeWatcher does not use Watchpack's ignored-glob matcher.
  if (matchesWindowsPath === undefined) {
    return;
  }
  expect(matchesWindowsPath).toBe(true);
});
