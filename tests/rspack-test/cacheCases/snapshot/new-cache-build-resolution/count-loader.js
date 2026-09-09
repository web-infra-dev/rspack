const fs = require('node:fs');
const path = require('node:path');
module.exports = function () {
  const counter = path.join(__dirname, '.loader-count');
  const runs = fs.existsSync(counter) ? Number(fs.readFileSync(counter, 'utf8')) + 1 : 1;
  fs.writeFileSync(counter, String(runs));
  return `module.exports = ${runs};`;
};
