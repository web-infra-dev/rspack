const modules = import.meta.glob('./*.JS', {
  eager: true,
  caseSensitive: 'yes',
})

it('should warn and default to case-sensitive matching for an invalid value', () => {
  expect(Object.keys(modules)).toEqual([])
})
