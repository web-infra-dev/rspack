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
    expect(fallbacks[0][0]).toBe(`independent-${CASE_NAME}/same_key/1.0.0/share-entry.js`);
    expect(fallbacks[0][2]).toBe(`equivalent_${CASE_NAME.replaceAll('-', '_')}_f_same_key_100`);
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
    if (CASE_NAME.startsWith('provider-')) {
      expect(shared.providers.map(({ version }) => version)).toEqual(['1.0.0', '2.0.0']);
      for (const provider of shared.providers) {
        const fallback = fallbacks.find(([, version]) => version === provider.version);
        expect(fallback).toBeDefined();
        expect(provider.fallback).toBe(fallback[0]);
        expect(provider.fallbackName).toBe(fallback[2]);
      }
    }
  }
});
