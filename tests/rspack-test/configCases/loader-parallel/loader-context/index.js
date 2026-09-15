it('should expose loader context APIs in parallel loaders', () => {
  expect(require('./resource')).toEqual({
    version: 2,
    logger: true,
    resolve: true,
    getResolve: true,
    path: 'path-hash.js',
    assetPath: '[asset-hash].js',
    pathWithInfo: 'info-hash-info-hash.js',
    pathWithInfoIdentity: true,
    pathWithInfoCustom: 'from-callback',
    pathWithInfoFullhash: true,
    assetPathWithInfo: 'asset-with-info.js',
    assetPathWithInfoCustom: 'from-asset-callback',
  });
});
