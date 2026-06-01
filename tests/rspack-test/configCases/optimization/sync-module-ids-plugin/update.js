const a = require('./a');

it('should update module ids by pruning unused records', () => {
  expect(a).toBe('a');

  const fs = __non_webpack_require__('fs');
  const path = __non_webpack_require__('path');
  const ids = JSON.parse(
    fs.readFileSync(path.join(__dirname, 'update-module-ids.json'), 'utf-8'),
  );

  expect(typeof ids['./a.js']).toBe('number');
  expect(typeof ids['./update.js']).toBe('number');
  expect(ids).not.toHaveProperty('./b.js');
  expect(ids).not.toHaveProperty('./c.js');
  expect(ids).not.toHaveProperty('./merge.js');

  const source = fs.readFileSync(
    path.join(__dirname, `bundle${__STATS_I__}.js`),
    'utf-8',
  );

  expect(source).toContain(`${ids['./a.js']}(module)`);
});
