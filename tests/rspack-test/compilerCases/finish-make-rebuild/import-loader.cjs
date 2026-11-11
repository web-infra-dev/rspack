const path = require('node:path');
module.exports = async function () {
  this.cacheable(false);
  const owner =
    path.basename(this.resourcePath) === 'imports.js' ? 'main' : 'other';
  if (owner === 'main' && this.rebuildState.drop) return 'export default -1;';
  const phase =
    owner === 'main' && this.rebuildState.versionedRequest
      ? this.rebuildState.phase
      : 100;
  const resource = owner === 'main' ? 'payload.js' : 'other-payload.js';
  const result = await this.importModule(
    `!!${path.join(__dirname, 'payload-loader.cjs')}?owner=${owner}&phase=${phase}!${path.join(__dirname, resource)}`,
  );
  return `export default ${JSON.stringify(result.default)};`;
};
