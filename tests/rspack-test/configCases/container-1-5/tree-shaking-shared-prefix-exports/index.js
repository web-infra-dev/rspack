const fs = require('fs');
const path = require('path');

it('collects exports under expanded prefix identities with normal request precedence', () => {
  const load = () => Promise.all([import('./consumer'), import('./server')]);
  expect(typeof load).toBe('function');
  const usedExports = __webpack_require__.federation.usedExports;
  const expected = {
    'custom-sub': ['manual', 'used'],
    'deep-sub': ['deepUsed'],
    exact: ['exactUsed'],
    'server-sub': ['manualServer', 'serverUsed'],
    'manual-long-sub': ['manualRight', 'overlapUsed'],
  };
  for (const [key, exports] of Object.entries(expected)) {
    expect(usedExports[key]).toEqual(exports);
  }
  expect(usedExports['server-deep/sub']).toBeUndefined();
  expect(usedExports['server-exact']).toBeUndefined();
  expect(usedExports['custom-disabled']).toBeUndefined();
  expect(usedExports['custom-disabled/sub']).toBeUndefined();

  for (const filename of ['mf-stats.json', 'mf-manifest.json']) {
    const manifest = JSON.parse(fs.readFileSync(path.join(__dirname, filename), 'utf-8'));
    expect(manifest.shared.find(shared => shared.name === 'directorysub.js').fallback).toEqual(expect.any(String));
    for (const [key, exports] of Object.entries(expected)) {
      expect(manifest.shared.find(shared => shared.name === key).usedExports).toEqual(exports);
    }
  }
});
