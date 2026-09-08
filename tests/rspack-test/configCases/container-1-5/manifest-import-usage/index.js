const fs = require('fs');
const path = require('path');

it('records each consuming import path and issuer layer in manifest stats', () => {
  const stats = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'mf-stats.json'), 'utf-8'),
  );
  const shared = stats.shared.find(({ name }) => name === 'shared');
  expect(shared.usedIn).toEqual(['second.js', 'third.js']);
  for (const name of ['multi', 'other']) {
    expect(stats.exposes.find((expose) => expose.name === name).requires).toEqual([
      'shared',
    ]);
  }
  const plain = stats.exposes.find(({ name }) => name === 'plain');
  expect(plain.requires).toEqual([]);
  expect(stats.exposes.find(({ name }) => name === 'multi').file).toBe('plain.js');
});
