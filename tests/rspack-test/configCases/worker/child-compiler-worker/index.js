const fs = require('fs');
const path = require('path');

it('should generate child entry when child compiler processes new Worker()', () => {
  const childEntryPath = path.resolve(__dirname, '__child-child-entry.js');
  expect(fs.existsSync(childEntryPath)).toBe(true);
});
