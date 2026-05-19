const path = require('node:path');

module.exports = function (code, map) {
  const expectedSource = path
    .resolve(__dirname, 'index.css')
    .replace(/\\/g, '/');

  expect(map).toBeTruthy();
  expect(map.sources).toEqual([expectedSource]);

  this.callback(null, code, map);
};
