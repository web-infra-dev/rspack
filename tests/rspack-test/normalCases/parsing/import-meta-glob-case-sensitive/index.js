const modules = import.meta.glob('./*.JS', {
  eager: true,
  caseSensitive: 'yes',
})
const staticallyEvaluatedModules = import.meta.glob('./case-test/*.JS', {
  eager: true,
  caseSensitive: !true,
})

it('should warn and default to case-sensitive matching for an invalid value', () => {
  expect(Object.keys(modules)).toEqual([])
})

it('should evaluate a static boolean caseSensitive expression', () => {
  expect(Object.keys(staticallyEvaluatedModules)).toEqual([
    './case-test/alpha.js',
  ])
})
