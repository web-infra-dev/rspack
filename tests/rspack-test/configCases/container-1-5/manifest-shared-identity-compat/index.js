const fs = require('fs');
const path = require('path');

const readJson = (name) =>
  JSON.parse(fs.readFileSync(path.join(__dirname, name), 'utf-8'));

const analyzedStats = readJson('analyzed-stats.json');
const analyzedManifest = readJson('analyzed.json');
const disabledStats = readJson('disabled-stats.json');
const disabledManifest = readJson('disabled.json');

const sharedIdentityFields = ({
  id,
  identityId,
  name,
  version,
  requiredVersion,
  layer,
  shareScope,
  singleton,
}) => ({
  id,
  identityId,
  name,
  version,
  requiredVersion,
  layer,
  shareScope,
  singleton,
});

const sortedSharedIdentities = (output) =>
  output.shared.map(sharedIdentityFields).sort((a, b) => a.id.localeCompare(b.id));

it('preserves legacy IDs and analyzed/disabled identity parity', () => {
  expect(sortedSharedIdentities(analyzedStats)).toEqual(
    sortedSharedIdentities(disabledStats),
  );
  expect(sortedSharedIdentities(analyzedManifest)).toEqual(
    sortedSharedIdentities(disabledManifest),
  );

  const legacy = analyzedStats.shared.find((shared) => shared.name === 'legacy');
  expect(legacy).toEqual(
    expect.objectContaining({
      id: 'container:legacy',
      shareScope: 'custom',
      version: '0',
    }),
  );

  const collisions = analyzedStats.shared.filter(
    (shared) => shared.name === 'collision',
  );
  expect(collisions).toHaveLength(2);
  expect(new Set(collisions.map((shared) => shared.id))).toEqual(
    new Set(['container:collision']),
  );
  expect(new Set(collisions.map((shared) => shared.identityId)).size).toBe(2);
  expect(
    collisions.every((shared) =>
      shared.identityId.startsWith('container:shared:'),
    ),
  ).toBe(true);
});

it('keeps exposes with the same import as separate public identities', () => {
  for (const output of [
    analyzedStats,
    analyzedManifest,
    disabledStats,
    disabledManifest,
  ]) {
    expect(output.exposes).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          id: 'container:first',
          path: './first',
          layer: 'first-layer',
        }),
        expect.objectContaining({
          id: 'container:second',
          path: './second',
          layer: 'second-layer',
        }),
      ]),
    );
  }
});

it('retains legacy requires and adds structured shared requirements', () => {
  for (const expose of analyzedStats.exposes) {
    expect(expose.requires).toContain('legacy');
    expect(expose.requiredShared).toContainEqual({
      name: 'legacy',
      shareScope: 'custom',
    });
  }
});
