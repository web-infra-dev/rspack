const fs = require('fs');
const path = require('path');

it('records scoped shared requirements through local helper modules and cycles', () => {
  for (const filename of ['mf-manifest.json', 'mf-stats.json']) {
    const output = JSON.parse(
      fs.readFileSync(path.join(__dirname, filename), 'utf-8'),
    );
    for (const name of ['first', 'second']) {
      const expose = output.exposes.find((expose) => expose.name === name);
      expect(expose.requiredShared).toEqual([
        { name: 'remote-only', shareScope: 'tenant', layer: 'shared' },
      ]);
      if (filename === 'mf-stats.json') {
        expect(expose.requires).toEqual(['remote-only']);
        expect(
          output.shared.find((share) => share.name === 'remote-only').usedIn,
        ).toContain(expose.file);
      }
    }
    const boundary = output.exposes.find(
      (expose) => expose.name === 'boundary',
    );
    expect(boundary.requiredShared).toEqual([
      { name: 'boundary-only', shareScope: 'tenant', layer: 'shared' },
    ]);
    if (filename === 'mf-stats.json')
      expect(boundary.requires).toEqual(['boundary-only']);
    const unrelated = output.exposes.find(
      (expose) => expose.name === 'unrelated',
    );
    expect(unrelated).not.toHaveProperty('requiredShared');
    if (filename === 'mf-stats.json') expect(unrelated.requires).toEqual([]);
  }
});
