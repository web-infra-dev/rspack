it('should keep fallback resources associated with their configured imports', () => {
  // Discover providers independently of the aliased consume requests.
  const loadProviders = () => Promise.all([
    import('pkg-a'),
    import('pkg-b'),
    import('alias-b'),
    import('pkg-a-copy'),
    import('pkg-a?copy'),
    import('pkg/sub'),
    import('mapped/a'),
  ]);
  expect(typeof loadProviders).toBe('function');

  const fallbacks = __webpack_require__.federation.sharedFallback;
  for (const [key, expected] of [
    ['same-key', [
      ['1.0.0', 'a'],
      ['1.0.0', 'a'],
      ['1.0.0', 'query'],
      ['2.0.0', 'b'],
      ['3.0.0', 'b'],
    ]],
    ['prefix/sub', [['1.0.0', 'sub']]],
    ['mapped/a', [['1.0.0', 'a']]],
  ]) {
    const actual = fallbacks[key].map(([entry, version, globalName]) => {
      const container = eval('require')(`./${entry}`)[globalName];
      return [version, container.get()().value];
    });
    expect(actual.sort()).toEqual(expected.sort());
  }
});
