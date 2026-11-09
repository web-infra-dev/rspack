const fs = require('fs');
const path = require('path');

it('keeps independent shared identities distinct and legacy names stable', async () => {
  const [{ value: valueA }, { value: valueB }, { value: legacyValue }] =
    await Promise.all([
      import('variant-a'),
      import('variant-b'),
      import('legacy-shared'),
    ]);
  expect([valueA, valueB, legacyValue]).toEqual(['a', 'b', 'legacy']);
  await Promise.all([
    import('default-unlayered'),
    import('default-layered'),
    import('custom-unlayered'),
  ]);

  const manifest = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'mf-manifest.json'), 'utf-8'),
  );
  const variants = manifest.shared.filter(
    ({ name, version }) => name === 'shared-variant' && version === '1.0.0',
  );
  expect(variants).toHaveLength(2);
  expect(new Set(variants.map(({ fallback }) => fallback)).size).toBe(2);
  expect(new Set(variants.map(({ fallbackName }) => fallbackName)).size).toBe(
    2,
  );
  for (const variant of variants) {
    expect(variant.fallback).toMatch(
      /^independent-packages\/shared_variant\/variant-[a-f0-9]{12}\/1\.0\.0\/share-entry\.js$/,
    );
    expect(fs.existsSync(path.join(__dirname, variant.fallback))).toBe(true);
  }

  const legacy = manifest.shared.find(
    ({ name, version }) => name === 'legacy-shared' && version === '1.0.0',
  );
  expect(legacy.shareScope).toEqual(['root', 'default']);
  expect(legacy.fallback).toBe(
    'independent-packages/legacy_shared/1.0.0/share-entry.js',
  );
  expect(fs.existsSync(path.join(__dirname, legacy.fallback))).toBe(true);

  const collisions = manifest.shared.filter(
    ({ name, version }) => name === 'default-collision' && version === '1.0.0',
  );
  expect(collisions).toHaveLength(3);
  expect(new Set(collisions.map(({ fallback }) => fallback)).size).toBe(3);
  expect(new Set(collisions.map(({ fallbackName }) => fallbackName)).size).toBe(
    3,
  );
  for (const collision of collisions) {
    expect(collision.fallback).toMatch(
      /^independent-packages\/default_collision\/variant-[a-f0-9]{12}\/1\.0\.0\/share-entry\.js$/,
    );
    expect(fs.existsSync(path.join(__dirname, collision.fallback))).toBe(true);
  }
});

it('does not copy custom-scope exports into an unoptimized default share', () => {
  const manifest = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'mf-manifest.json'), 'utf-8'),
  );
  const shares = manifest.shared.filter(({ name }) => name === 'unoptimized-collision');
  expect(shares).toHaveLength(2);
  expect(shares.find(({ shareScope }) => shareScope === undefined)).not.toHaveProperty('usedExports');
  expect(shares.find(({ shareScope }) => shareScope === 'custom').usedExports).toContain('value');
});

it('retains provider scope and layer in the runtime full-fallback table', () => {
  const fallbacks = __webpack_require__.federation.sharedFallback;
  expect(fallbacks['shared-variant'].map(([, version, , identity]) => ({ version, ...identity }))).toEqual([
    { version: '1.0.0', shareScope: ['scope-a', 'default'], layer: 'layer-a' },
    { version: '1.0.0', shareScope: ['scope-b', 'default'], layer: 'layer-b' },
  ]);
  expect(fallbacks['default-collision'].map(([, , , identity]) => identity)).toEqual([
    { shareScope: 'default' },
    { shareScope: 'default', layer: 'layered' },
    { shareScope: 'custom' },
  ]);
});
