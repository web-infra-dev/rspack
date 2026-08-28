it('should synchronously call option functions on the main thread', () => {
  expect(require('./value')).toEqual({
    value: 'prefix:value',
    nested: 'worker:value',
    thrown: 'function option failed',
    map: 'structured-clone',
    typed: [1, 2, 3],
    url: 'https://rspack.dev/loader-options',
    custom: 'custom:context',
    hook: 'hook:context',
    hookMainThread: true,
    worker: true,
  });
});

it('should preserve the original options object in the main isolate', () => {
  expect(require('./main')).toBe(true);
});
