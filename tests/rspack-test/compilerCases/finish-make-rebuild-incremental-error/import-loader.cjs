const path = require('node:path');

module.exports = async function () {
  const result = await this.importModule(path.join(this.context, 'payload.js'));
  this.emitFile('stable.txt', result.default);
  return `export default ${JSON.stringify(result.default)};`;
};
