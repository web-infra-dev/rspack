const fs = require('fs');
const path = require('path');

const discoverProvider = () => import('pkg-a');

it('emits one artifact per resolved build configuration', () => {
  expect(typeof discoverProvider).toBe('function');
  const fallbacks = __webpack_require__.federation.sharedFallback['same-key'];
  expect(fallbacks).toHaveLength(EXPECTED_ARTIFACTS);
  expect(new Set(fallbacks.map(([entry]) => entry)).size).toBe(EXPECTED_ARTIFACTS);
  expect(new Set(fallbacks.map(([, , globalName]) => globalName)).size).toBe(EXPECTED_ARTIFACTS);
  if (EXPECTED_ARTIFACTS === 1) {
    const [outputDir, shareDir, ...file] = fallbacks[0][0].split('/');
    expect(outputDir).toBe(`independent-${CASE_NAME}`);
    expect(shareDir).toMatch(/^mf_[0-9a-f]{16}$/);
    expect(file.join('/')).toBe('1.0.0/share-entry.js');
    expect(fallbacks[0][2]).toMatch(/^mf_[0-9a-f]{16}$/);
  }
  for (const [entry, version, globalName] of fallbacks) {
    expect(CASE_NAME.startsWith('provider-') ? ['1.0.0', '2.0.0'] : ['1.0.0']).toContain(version);
    const container = eval('require')(`./${entry}`)[globalName];
    expect(container.get()().value).toBe('a');
  }
});

it('attaches fallback metadata when the resolved artifact is unambiguous', () => {
  const fallbacks = __webpack_require__.federation.sharedFallback['same-key'];
  for (const filename of [`${CASE_NAME}-stats.json`, `${CASE_NAME}.json`]) {
    const output = JSON.parse(fs.readFileSync(path.join(__dirname, filename), 'utf-8'));
    const [shared] = output.shared.filter(({ name }) => name === 'same-key');
    expect(shared).toBeDefined();
    if (!CASE_NAME.startsWith('different-')) {
      const fallback = fallbacks.find(([, version]) => version === shared.version);
      expect(fallback).toBeDefined();
      expect(shared.fallback).toBe(fallback[0]);
      expect(shared.fallbackName).toBe(fallback[2]);
    } else {
      expect(shared).not.toHaveProperty('fallback');
      expect(shared).not.toHaveProperty('fallbackName');
    }
    expect(shared).not.toHaveProperty('providers');
  }
});
