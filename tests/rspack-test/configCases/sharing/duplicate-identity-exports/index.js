const fs = require('fs');
const path = require('path');
it('unions configured exports for providers with the same shared identity', () => {
  const stats = JSON.parse(fs.readFileSync(path.join(__dirname, 'mf-stats.json'), 'utf8'));
  expect(stats.shared.find(item => item.name === 'duplicate').usedExports).toEqual(['a', 'b']);
});
