const fs = require('fs');
const path = require('path');

const readJson = (name) =>
  JSON.parse(fs.readFileSync(path.join(__dirname, '..', name), 'utf8'));

it('records concrete providers without changing legacy shared fields', () => {
  for (const suffix of ['', '-stats']) {
    const output = readJson(`analyzed${suffix}.json`);
    const shared = output.shared.find((item) => item.name === 'multiple');
    expect(shared.id).toBe('shared_providers:multiple');
    expect(
      shared.providers.map(({ version, import: request }) => [version, request]),
    ).toEqual([
      ['1.0.0', './first.js'],
      ['2.0.0', './second.js'],
    ]);
    for (const provider of shared.providers) {
      expect(provider.assets.js.sync.length).toBeGreaterThan(0);
      for (const asset of provider.assets.js.sync) {
        expect(fs.existsSync(path.join(__dirname, '..', asset))).toBe(true);
      }
    }
    for (const name of ['single', 'consumer-only']) {
      expect(output.shared.find((item) => item.name === name)).not.toHaveProperty(
        'providers',
      );
    }
    for (const item of readJson(`disabled${suffix}.json`).shared) {
      expect(item).not.toHaveProperty('providers');
    }
  }
});
