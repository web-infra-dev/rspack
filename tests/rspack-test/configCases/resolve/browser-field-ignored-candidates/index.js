it('continues resolving after a missing file or path', () => {
  expect(require('missing-first')).toBe('fallback');
  expect(require('./literal.js#fragment')).toBe('base file');
  expect(require('local/index.js')).toBe('package');
  expect(require('pkg/index.js')).toBe('index');
});

it('stops at the first candidate ignored by the browser field', () => {
  expect({
    main: require('pkg'),
    preferRelative: require('local'),
    literalHash: require('./literal.js#ignored'),
    tsconfigPaths: require('mapped'),
    importsArray: require('#array'),
  }).toEqual({
    main: {},
    preferRelative: {},
    literalHash: {},
    tsconfigPaths: {},
    importsArray: {},
  });
});
