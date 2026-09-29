const fs = require('node:fs');
const path = require('node:path');
const dependencies = {
  immutable: 'immutable/data.txt',
  sibling: 'immutable-copy/data.txt',
  override: 'immutable/mutable/data.txt',
  managed: 'packages/pkg/internal/data.txt',
  scoped: 'packages/@scope/pkg/data.txt',
  nested: 'packages/pkg/node_modules/nested/data.txt',
  context: 'packages/unversioned/data.txt',
};
module.exports = function () {
  const name = path.basename(this.resourcePath, '.js').slice('consumer-'.length);
  const dependency = path.join(__dirname, dependencies[name]);
  if (name === 'context') this.addContextDependency(path.dirname(dependency));
  else this.addDependency(dependency);
  return `module.exports = ${JSON.stringify(fs.readFileSync(dependency, 'utf8').trim())};`;
};
