const fs = require('fs');
const path = require('path');

it('preserves every explicit provider version without guessing an unknown consumer version', () => {
  for (const name of ['forward', 'reverse']) {
    for (const suffix of ['', '-stats']) {
      const output = JSON.parse(
        fs.readFileSync(
          path.join(__dirname, '..', name + suffix + '.json'),
          'utf-8',
        ),
      );
      const versions = output.shared.filter(
        (share) => share.name === 'versions',
      );
      expect(versions.map((share) => share.version).sort()).toEqual([
        '',
        '1.0.0',
        '2.0.0',
      ]);
      for (const share of versions) {
        expect(share).not.toHaveProperty('shareScope');
        expect(share.assets).toEqual({
          js: { sync: [], async: [] },
          css: { sync: [], async: [] },
        });
      }
      const unique = output.shared.filter((share) => share.name === 'unique');
      expect(unique).toHaveLength(1);
      expect(unique[0].version).toBe('1.0.0');
      expect(unique[0]).not.toHaveProperty('shareScope');
    }
  }
});

it('emits default scope only for rows that collide in name, version and layer', () => {
  for (const name of ['forward', 'reverse']) {
    for (const suffix of ['', '-stats']) {
      const output = JSON.parse(
        fs.readFileSync(
          path.join(__dirname, '..', name + suffix + '.json'),
          'utf-8',
        ),
      );
      const shares = output.shared.filter((share) => share.name === 'scopes');
      expect(shares).toHaveLength(4);
      const siblings = shares.filter(
        (share) => share.version === '1.0.0' && !share.layer,
      );
      expect(siblings.map((share) => share.shareScope).sort()).toEqual([
        'custom',
        'default',
      ]);
      expect(
        shares.find((share) => share.version === '2.0.0'),
      ).not.toHaveProperty('shareScope');
      expect(
        shares.find((share) => share.layer === 'server'),
      ).not.toHaveProperty('shareScope');
    }
  }
});
