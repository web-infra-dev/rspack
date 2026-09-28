const fs = require('fs');
const path = require('path');

it('keeps manual mock exports alive for a literal rs.requireMock', () => {
  const content = fs.readFileSync(
    path.resolve(__dirname, 'usedExports.mjs'),
    'utf-8',
  );

  expect(content).toContain('other: other');
  expect(content).toContain('value: value');
});
