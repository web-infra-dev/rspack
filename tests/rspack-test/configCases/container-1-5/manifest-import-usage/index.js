const fs = require('fs');
const path = require('path');

it('records each consuming import path and issuer layer in manifest stats', () => {
  const stats = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'mf-stats.json'), 'utf-8'),
  );
  const shared = stats.shared.find(({ name }) => name === 'shared');
  expect(shared.usedIn).toEqual([
    stats.exposes.find(({ name }) => name === 'direct').file,
    'second.js',
    'third.js',
  ].sort());
  for (const name of ['multi', 'other']) {
    expect(stats.exposes.find((expose) => expose.name === name).requires).toEqual([
      'shared',
    ]);
  }
  const plain = stats.exposes.find(({ name }) => name === 'plain');
  expect(plain.requires).toEqual([]);
  expect(stats.exposes.find(({ name }) => name === 'multi').file).toBe('plain.js');
});

it('links directly exposed shares, including multi-import and layered exposes', () => {
  const stats = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'mf-stats.json'), 'utf-8'),
  );
  const direct = stats.exposes.find(({ name }) => name === 'direct');
  const multi = stats.exposes.find(({ name }) => name === 'direct-multi');
  const layered = stats.exposes.find(({ name }) => name === 'direct-layered');
  const consumeOnly = stats.exposes.find(({ name }) => name === 'consume-only');
  expect(consumeOnly.requires).toEqual(['consume-only']);
  expect(stats.shared.find(({ name }) => name === 'consume-only').usedIn).toEqual([
    consumeOnly.file,
  ]);
  expect(direct.requires).toEqual(['shared']);
  expect(direct).not.toHaveProperty('requiredShared');
  expect(multi.requires).toEqual(['shared']);
  expect(multi.file).toBe('plain.js');
  const shared = stats.shared.find(({ name }) => name === 'shared');
  expect(shared.usedIn).toContain(direct.file);
  expect(shared.usedIn).not.toContain('plain.js');
  expect(layered.requires).toEqual(['shared-layered']);
  expect(layered.requiredShared).toEqual([
    { name: 'shared-layered', layer: 'shared-layer', shareScope: 'custom' },
  ]);
  expect(stats.shared.find(({ name }) => name === 'shared-layered').usedIn).toEqual([
    layered.file,
  ]);
});
