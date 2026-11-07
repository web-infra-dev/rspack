const fs = require('fs');
const path = require('path');

it('should resolve a layered relative fallback from the issuer context', async () => {
  const value = await import('./nested/consumer');
  expect(value.default).toBe('issuer-relative-shared');
});

it('should retain the layered consume in federation stats', () => {
  const stats = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'mf-stats.json'), 'utf-8'),
  );
  expect(
    stats.shared.find((shared) => shared.name === 'relative-shared'),
  ).toBeDefined();
});
