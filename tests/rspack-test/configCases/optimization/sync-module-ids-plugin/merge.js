const a = require('./a');
const b = require('./b');
const c = require('./c');

it('should merge module ids with existing records', () => {
  expect(a + b + c).toBe('abc');

  const fs = require('fs');
  const path = require('path');
  const ids = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'merge-module-ids.json'), 'utf-8'),
  );

  expect(typeof ids['./a.js']).toBe('string');
  expect(typeof ids['./b.js']).toBe('string');
  expect(typeof ids['./c.js']).toBe('string');
  expect(typeof ids['./merge.js']).toBe('string');
  expect(typeof ids['./seed.js']).toBe('string');

  const source = fs.readFileSync(
    path.join(__dirname, `bundle${__STATS_I__}.js`),
    'utf-8',
  );

  expect(source).toContain(`${JSON.stringify(ids['./a.js'])}(module)`);
  expect(source).toContain(`${JSON.stringify(ids['./b.js'])}(module)`);
  expect(source).toContain(`${JSON.stringify(ids['./c.js'])}(module)`);
});
