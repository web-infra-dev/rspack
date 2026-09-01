const fs = require('fs');
const path = require('path');

module.exports = function () {
  const dependency = path.join(__dirname, 'build-dependency.js');
  this.addBuildDependency(dependency);
  return `export default ${JSON.stringify(fs.readFileSync(dependency, 'utf-8').trim())};`;
};
