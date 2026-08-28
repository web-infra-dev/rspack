/** @type {import('@rspack/core').LoaderDefinition} */
module.exports = function () {
  const callback = this.async();
  const logger = this.getLogger('parallel-loader');
  const getResolve = this.getResolve();
  const fsContent = this.fs.readFileSync(this.resourcePath).toString();
  const assetPath = this._compilation.getPath('assets/[id].js', {
    id: 'parallel',
  });
  const module = this._module;
  module.buildInfo.parallelAccessor = 'native';

  logger.clear();
  logger.info('loader context APIs are available');

  this.resolve(
    this.context,
    './dependency.js',
    (resolveError, _resolveResult, resolveRequest) => {
      if (resolveError) {
        callback(resolveError);
        return;
      }

      getResolve(
        this.context,
        './dependency.js',
        (getResolveError, _getResolveResult, getResolveRequest) => {
          if (getResolveError) {
            callback(getResolveError);
            return;
          }

          callback(
            null,
            `module.exports = ${JSON.stringify({
              version: this.version,
              logger: typeof logger.clear === 'function',
              resolve: resolveRequest?.path.endsWith('dependency.js'),
              getResolve: getResolveRequest?.path.endsWith('dependency.js'),
              fs: fsContent.includes("export default 'resource'"),
              path: assetPath,
              module: {
                identifier: module.identifier().includes('resource.js'),
                readableIdentifier: module.readableIdentifier().includes('resource.js'),
                nameForCondition: module.nameForCondition().endsWith('resource.js'),
                resource: module.resource.endsWith('resource.js'),
                request: module.request.includes('resource.js'),
                userRequest: module.userRequest.endsWith('resource.js'),
                rawRequest: typeof module.rawRequest === 'string' && module.rawRequest.length > 0,
                resourceResolveData: module.resourceResolveData.path.endsWith('resource.js'),
                loaders: module.loaders.length === 1,
                buildInfo: module.buildInfo.parallelAccessor === 'native',
              },
            })}`,
          );
        },
      );
    },
  );
};
