import first from 'first';
import second from 'second';
const fs = require('fs');
const path = require('path');
it('keeps delimiter-containing shared identities distinct', () => {
  expect(first).toBe(42);
  expect(second).toBe(42);
  const modules = JSON.parse(fs.readFileSync(path.join(__dirname, 'identities.json'), 'utf8'));
  for (const kind of ['provide shared module', 'consume shared module']) {
    const selected = modules.filter(({ id }) => id.startsWith(kind));
    expect(selected).toHaveLength(2);
    expect(new Set(selected.map(({ id }) => id)).size).toBe(2);
    expect(new Set(selected.map(({ name }) => name)).size).toBe(1);
  }
});
