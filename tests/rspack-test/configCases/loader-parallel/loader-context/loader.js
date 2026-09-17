/** @type {import('@rspack/core').LoaderDefinition} */
module.exports = function () {
  const callback = this.async();
  const logger = this.getLogger('parallel-loader');
  const getResolve = this.getResolve();
  const compilation = this._compilation;

  const path = compilation.getPath((data) => {
    data.hash = 'path-hash';
    return '[fullhash].js';
  });
  const assetPath = compilation.getAssetPath(
    ({ hash }) => `[${hash}].js`,
    { hash: 'asset-hash' },
  );

  let callbackInfo;
  const pathWithInfo = compilation.getPathWithInfo(
    ({ hash }, info) => {
      callbackInfo = info;
      info.custom = 'from-callback';
      info.fullhash = 'from-callback';
      return `[fullhash]-${hash}.js`;
    },
    { hash: 'info-hash' },
  );
  const assetPathWithInfo = compilation.getAssetPathWithInfo(
    (_data, info) => {
      info.custom = 'from-asset-callback';
      return 'asset-with-info.js';
    },
  );

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
              path,
              assetPath,
              pathWithInfo: pathWithInfo.path,
              pathWithInfoIdentity: pathWithInfo.info === callbackInfo,
              pathWithInfoCustom: pathWithInfo.info.custom,
              pathWithInfoFullhash:
                new Set(pathWithInfo.info.fullhash).size === 2 &&
                pathWithInfo.info.fullhash.includes('from-callback') &&
                pathWithInfo.info.fullhash.includes('info-hash'),
              assetPathWithInfo: assetPathWithInfo.path,
              assetPathWithInfoCustom: assetPathWithInfo.info.custom,
            })}`,
          );
        },
      );
    },
  );
};
