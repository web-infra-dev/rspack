const path = require('node:path');
module.exports = async function () {
  const { depth } = this.getOptions();
  if (Number(depth) === 0) return 'export default 42;';
  const result = await this.importModule(
    `!!${__filename}?depth=${Number(depth) - 1}!${path.join(__dirname, 'payload.js')}`,
  );
  return `export default ${result.default + 1};`;
};
