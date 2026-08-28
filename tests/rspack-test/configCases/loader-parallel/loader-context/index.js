it('should expose loader context APIs in parallel loaders', () => {
  expect(require('./resource')).toEqual({
    version: 2,
    logger: true,
    resolve: true,
    getResolve: true,
    fs: true,
    path: 'assets/parallel.js',
    module: {
      identifier: true,
      readableIdentifier: true,
      nameForCondition: true,
      resource: true,
      request: true,
      userRequest: true,
      rawRequest: true,
      resourceResolveData: true,
      loaders: true,
      buildInfo: true,
    },
  });
});

it('should use the main-thread input filesystem bridge', () => {
  expect(require('./stats')).toBe(true);
});
