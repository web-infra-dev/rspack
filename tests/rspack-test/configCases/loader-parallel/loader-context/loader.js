/** @type {import('@rspack/core').LoaderDefinition} */
module.exports = function () {
  const callback = this.async();
  const logger = this.getLogger('parallel-loader');
  const getResolve = this.getResolve();

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
            })}`,
          );
        },
      );
    },
  );
};
