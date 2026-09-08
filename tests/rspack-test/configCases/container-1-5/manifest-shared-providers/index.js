const fs = require('fs');
const path = require('path');

it('retains one shared row with concrete provider versions, imports and assets', () => {
  for (const name of ['layered', 'default']) {
    for (const filename of [`${name}-stats.json`, `${name}.json`]) {
      const output = JSON.parse(
        fs.readFileSync(path.join(__dirname, '..', filename), 'utf-8'),
      );
      const shares = output.shared.filter(({ name }) => name === 'multiple');
      expect(shares).toHaveLength(1);
      const [shared] = shares;
      if (name === 'layered') {
        expect(shared).toEqual(
          expect.objectContaining({ shareScope: 'custom', layer: 'server' }),
        );
      } else {
        expect(shared.id).toBe('shared_providers:multiple');
        expect(shared).not.toHaveProperty('layer');
        expect(shared).not.toHaveProperty('shareScope');
        expect(shared).not.toHaveProperty('identityId');
      }
      expect(['1.0.0', '2.0.0']).toContain(shared.version);
      expect(
        shared.providers.map(({ version, import: request }) => [version, request]),
      ).toEqual([
        ['1.0.0', './first.js'],
        ['2.0.0', './second.js'],
      ]);
      const aggregateFiles = [...shared.assets.js.sync, ...shared.assets.js.async];
      for (const [index, provider] of shared.providers.entries()) {
        expect(provider.fallback).toEqual(expect.any(String));
        expect(provider.fallbackName).toEqual(expect.any(String));
        expect(fs.existsSync(path.join(__dirname, '..', provider.fallback))).toBe(true);
        const files = [...provider.assets.js.sync, ...provider.assets.js.async];
        expect(files.length).toBeGreaterThan(0);
        expect(aggregateFiles).toEqual(expect.arrayContaining(files));
        const source = files
          .map((file) => fs.readFileSync(path.join(__dirname, '..', file), 'utf-8'))
          .join('\n');
        expect(source).toContain(index === 0 ? 'provider-first' : 'provider-second');
        expect(source).not.toContain(index === 0 ? 'provider-second' : 'provider-first');
      }
      expect(output.shared.find(({ name }) => name === 'single')).not.toHaveProperty(
        'providers',
      );
      const consumer = output.shared.find(({ name }) => name === 'consumer-only');
      expect(consumer).toBeDefined();
      expect(consumer).not.toHaveProperty('providers');
    }
  }
});
