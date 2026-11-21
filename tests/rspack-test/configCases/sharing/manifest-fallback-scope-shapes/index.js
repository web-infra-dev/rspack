const fs = require('fs');
const path = require('path');

// Keep both configured import paths in the graph without choosing a runtime provider.
module.exports = [
  () => import('one'),
  () => import('two'),
  () => import('ambiguous-a'),
  () => import('ambiguous-b'),
];

it('keeps both exact scalar and array scope fallbacks in manifest and stats', async () => {
  for (const filename of ['mf-manifest.json', 'mf-stats.json']) {
    const output = JSON.parse(
      fs.readFileSync(path.join(__dirname, filename), 'utf-8'),
    );
    const shares = output.shared.filter((share) => share.name === 'shape');
    expect(shares).toHaveLength(2);
    expect(
      shares.find((share) => typeof share.shareScope === 'string').shareScope,
    ).toBe('default');
    expect(
      shares.find((share) => Array.isArray(share.shareScope)).shareScope,
    ).toEqual(['default']);
    expect(new Set(shares.map((share) => share.fallback)).size).toBe(2);
    expect(new Set(shares.map((share) => share.fallbackName)).size).toBe(2);
    for (const share of shares) {
      expect(typeof share.fallback).toBe('string');
      expect(typeof share.fallbackName).toBe('string');
      expect(fs.existsSync(path.join(__dirname, share.fallback))).toBe(true);
      const container = __non_webpack_require__('./' + share.fallback)[
        share.fallbackName
      ];
      const factory = await container.get();
      expect(factory()).toBe(Array.isArray(share.shareScope) ? 'two' : 'one');
    }
  }
});

it('does not guess a fallback when two imports have the same structural identity', () => {
  for (const filename of ['mf-manifest.json', 'mf-stats.json']) {
    const output = JSON.parse(
      fs.readFileSync(path.join(__dirname, filename), 'utf-8'),
    );
    const shares = output.shared.filter((share) => share.name === 'ambiguous');
    expect(shares).toHaveLength(1);
    expect(shares[0]).not.toHaveProperty('fallback');
    expect(shares[0]).not.toHaveProperty('fallbackName');
    expect(shares[0]).not.toHaveProperty('shareScope');
  }
});
