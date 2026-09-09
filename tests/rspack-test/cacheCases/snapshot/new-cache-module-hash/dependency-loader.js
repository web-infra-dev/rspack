const fs = require('node:fs');
const path = require('node:path');
module.exports = function () {
  const dependency = path.join(__dirname, 'data.txt');
  const counter = path.join(__dirname, '.loader-count');
  this.addDependency(dependency);
  const runs = fs.existsSync(counter) ? Number(fs.readFileSync(counter, 'utf8')) + 1 : 1;
  fs.writeFileSync(counter, String(runs));
  return `module.exports = ${JSON.stringify({ value: fs.readFileSync(dependency, 'utf8').trim(), runs })}`;
};
