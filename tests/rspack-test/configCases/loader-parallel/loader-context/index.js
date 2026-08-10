it('should expose loader context APIs in parallel loaders', () => {
  expect(require('./resource')).toEqual({
    version: 2,
    logger: true,
    resolve: true,
    getResolve: true,
  });
});
