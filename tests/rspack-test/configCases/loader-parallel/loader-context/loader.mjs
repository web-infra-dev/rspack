import assert from 'node:assert/strict';
import { join } from 'node:path';

/** @type {import('@rspack/core').LoaderDefinition} */
export default function () {
  const file = join(this.context, 'dependency.js');
  const missing = join(this.context, 'missing.js');
  this.dependency(file);
  this.addContextDependency(this.context);
  this.addMissingDependency(missing);
  this.addBuildDependency(file);
  assert.ok(this.getDependencies().includes(file));
  assert.ok(this.getContextDependencies().includes(this.context));
  assert.ok(this.getMissingDependencies().includes(missing));

  this.clearDependencies();
  this.addDependency(this.resourcePath);
  assert.deepEqual(this.getDependencies(), [this.resourcePath]);
  assert.deepEqual(this.getContextDependencies(), []);
  assert.deepEqual(this.getMissingDependencies(), []);

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
  const secondLogger = this.getLogger('parallel-loader');
  logger.time('work');
  secondLogger.time('work');
  secondLogger.timeEnd('work');
  logger.timeEnd('work');

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
